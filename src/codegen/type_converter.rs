use super::error::{CodeGenError, CodeGenResult};
use crate::ast::{OwnershipType, Type};
use inkwell::{
    context::Context,
    types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, StructType},
    values::{BasicValue, BasicValueEnum},
    AddressSpace,
};
use std::collections::HashMap;

/// Handles type conversions between Replica's type system and LLVM types
pub struct TypeConverter<'ctx> {
    context: &'ctx Context,
    struct_types: HashMap<String, StructType<'ctx>>,
    cached_types: HashMap<String, BasicTypeEnum<'ctx>>,
}

impl<'ctx> TypeConverter<'ctx> {
    /// Creates a new TypeConverter instance
    pub fn new(context: &'ctx Context) -> Self {
        TypeConverter {
            context,
            struct_types: HashMap::new(),
            cached_types: HashMap::new(),
        }
    }

    /// Registers a custom struct type
    pub fn register_struct_type(&mut self, name: &str, struct_type: StructType<'ctx>) {
        self.struct_types.insert(name.to_string(), struct_type);
    }

    /// Converts a Replica type to an LLVM basic type
    pub fn convert_to_llvm(&self, ty: &Type) -> CodeGenResult<BasicTypeEnum<'ctx>> {
        match ty {
            Type::Int => Ok(self.context.i32_type().as_basic_type_enum()),
            Type::Float => Ok(self.context.f64_type().as_basic_type_enum()),
            Type::String => {
                // 文字列は文字配列へのポインタとして扱う
                Ok(self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum())
            }
            Type::Bool => Ok(self.context.bool_type().as_basic_type_enum()),
            Type::Custom(name) => self.get_custom_type(name),
            Type::Array(element_type) => {
                // 配列は要素型へのポインタとして実装
                let elem_type = self.convert_to_llvm(element_type)?;
                Ok(elem_type
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum())
            }
            Type::Optional(inner_type) => {
                // Optional型は内部型とbooleanフラグの構造体として実装
                self.create_optional_type(inner_type)
            }
        }
    }

    /// Converts a Replica type to an LLVM metadata type
    pub fn convert_to_metadata(&self, ty: &Type) -> CodeGenResult<BasicMetadataTypeEnum<'ctx>> {
        self.convert_to_llvm(ty).map(Into::into)
    }

    /// Creates a default value for a given type
    pub fn create_default_value(&self, ty: &Type) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match ty {
            Type::Int => Ok(self.context.i32_type().const_zero().as_basic_value_enum()),
            Type::Float => Ok(self.context.f64_type().const_zero().as_basic_value_enum()),
            Type::Bool => Ok(self.context.bool_type().const_zero().as_basic_value_enum()),
            Type::String => {
                // 空文字列のための定数を作成
                Ok(self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .as_basic_value_enum())
            }
            Type::Custom(name) => self.create_default_custom_value(name),
            Type::Array(_) => {
                // null ポインタを返す
                Ok(self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .as_basic_value_enum())
            }
            Type::Optional(_) => {
                // None値を表す0を返す
                Ok(self.context.i32_type().const_zero().as_basic_value_enum())
            }
        }
    }

    /// Gets the size of a type in bytes
    pub fn get_type_size(&self, ty: &Type) -> CodeGenResult<u32> {
        let llvm_type = self.convert_to_llvm(ty)?;
        Ok(llvm_type
            .size_of()
            .unwrap_or_else(|| self.context.i32_type().size_of())
            .const_bit_cast(self.context.i32_type())
            .get_zero_extended_constant()
            .unwrap_or(4) as u32)
    }

    /// Checks if a type is copyable
    pub fn is_copyable(&self, ty: &Type) -> bool {
        match ty {
            Type::Int | Type::Float | Type::Bool => true,
            Type::String => false,    // 文字列は所有権を持つ
            Type::Custom(_) => false, // カスタム型はデフォルトでコピー不可
            Type::Array(_) => false,  // 配列は所有権を持つ
            Type::Optional(inner) => self.is_copyable(inner),
        }
    }

    // Private helper methods
    fn get_custom_type(&self, name: &str) -> CodeGenResult<BasicTypeEnum<'ctx>> {
        self.struct_types
            .get(name)
            .map(|st| st.as_basic_type_enum())
            .ok_or_else(|| CodeGenError::TypeConversion(format!("Unknown custom type: {}", name)))
    }

    fn create_optional_type(&self, inner_type: &Type) -> CodeGenResult<BasicTypeEnum<'ctx>> {
        let inner_llvm_type = self.convert_to_llvm(inner_type)?;
        let fields = vec![
            inner_llvm_type,
            self.context.bool_type().as_basic_type_enum(),
        ];

        Ok(self
            .context
            .struct_type(&fields, false)
            .as_basic_type_enum())
    }

    fn create_default_custom_value(&self, name: &str) -> CodeGenResult<BasicValueEnum<'ctx>> {
        self.struct_types
            .get(name)
            .map(|st| {
                st.ptr_type(AddressSpace::default())
                    .const_null()
                    .as_basic_value_enum()
            })
            .ok_or_else(|| CodeGenError::TypeConversion(format!("Unknown custom type: {}", name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> Context {
        Context::create()
    }

    #[test]
    fn test_primitive_type_conversion() {
        let context = create_test_context();
        let converter = TypeConverter::new(&context);

        assert!(converter.convert_to_llvm(&Type::Int).is_ok());
        assert!(converter.convert_to_llvm(&Type::Float).is_ok());
        assert!(converter.convert_to_llvm(&Type::Bool).is_ok());
    }

    #[test]
    fn test_string_type_conversion() {
        let context = create_test_context();
        let converter = TypeConverter::new(&context);

        let result = converter.convert_to_llvm(&Type::String);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BasicTypeEnum::PointerType(_)));
    }

    #[test]
    fn test_array_type_conversion() {
        let context = create_test_context();
        let converter = TypeConverter::new(&context);

        let array_type = Type::Array(Box::new(Type::Int));
        let result = converter.convert_to_llvm(&array_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optional_type_conversion() {
        let context = create_test_context();
        let converter = TypeConverter::new(&context);

        let optional_type = Type::Optional(Box::new(Type::Int));
        let result = converter.convert_to_llvm(&optional_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_type_handling() {
        let context = create_test_context();
        let mut converter = TypeConverter::new(&context);

        // カスタム構造体型を登録
        let struct_type = context.struct_type(&[], false);
        converter.register_struct_type("MyStruct", struct_type);

        // 変換をテスト
        let result = converter.convert_to_llvm(&Type::Custom("MyStruct".to_string()));
        assert!(result.is_ok());
    }
}
