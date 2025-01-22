use inkwell::{
    context::Context,
    module::Module,
    builder::Builder,
    values::{FunctionValue, BasicValueEnum, BasicValue, IntValue, PointerValue},
    types::{BasicTypeEnum, BasicMetadataTypeEnum},
    targets::{
        Target,
        InitializationConfig,
        CodeModel,
        RelocMode,
        FileType,
    },
    AddressSpace,
    OptimizationLevel,
    targets::TargetTriple,
};
use thiserror::Error;
use crate::ast::*;
use std::collections::HashMap;
use inkwell::types::BasicType;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("Type conversion error: {0}")]
    TypeConversion(String),
    #[error("Compilation error: {0}")]
    Compilation(String),
    #[error("WASM generation error: {0}")]
    WasmGen(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
}

pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    actor_methods: HashMap<String, FunctionValue<'ctx>>,
    variables: HashMap<String, BasicValueEnum<'ctx>>,
    struct_types: HashMap<String, inkwell::types::StructType<'ctx>>,
    debug_mode: bool,
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Target::initialize_webassembly(&InitializationConfig::default());

        CodeGenerator {
            context,
            module,
            builder,
            actor_methods: HashMap::new(),
            variables: HashMap::new(),
            struct_types: HashMap::new(),
            debug_mode: false,
        }
    }

    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
    }

    fn debug_log(&self, message: &str) {
        if self.debug_mode {
            eprintln!("[CodeGen Debug] {}", message);
        }
    }

    pub fn compile_actor(&mut self, actor: &Actor) -> Result<(), CodeGenError> {
        self.debug_log(&format!("Compiling actor: {}", actor.name));

        // アクター型の作成
        let struct_type = self.context.opaque_struct_type(&actor.name);

        // フィールドの型を収集
        let field_types: Vec<BasicTypeEnum> = actor
            .fields
            .iter()
            .map(|field| self.convert_type_to_llvm(&field.field_type))
            .collect::<Result<Vec<_>, CodeGenError>>()?;  // CodeGenErrorを使用

        struct_type.set_body(&field_types, false);
        self.struct_types.insert(actor.name.clone(), struct_type);

        // メソッドのコンパイル
        for method in &actor.methods {
            self.debug_log(&format!("Compiling method: {}", method.name));
            let function = self.compile_method(method, &struct_type)
                .map_err(|e| CodeGenError::Compilation(format!("Failed to compile method {}: {}", method.name, e)))?;
            self.actor_methods.insert(method.name.clone(), function);
        }

        Ok(())
    }

    fn compile_method(
        &mut self,
        method: &Method,
        struct_type: &inkwell::types::StructType<'ctx>,
    ) -> Result<FunctionValue<'ctx>, CodeGenError> {
        self.variables.clear();

        // パラメータ型の変換
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = method
            .params
            .iter()
            .map(|param| self.convert_type_to_metadata(&param.param_type))
            .collect::<Result<Vec<_>, CodeGenError>>()?;

        // 関数型の作成
        let fn_type = match &method.return_type {
            Some(ty) => {
                let return_type = self.convert_type_to_llvm(ty)?;
                return_type.fn_type(&param_types, false)
            },
            None => self.context.void_type().fn_type(&param_types, false),
        };

        let function = self.module.add_function(&method.name, fn_type, None);

        // エントリーブロックの作成
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        // パラメータの登録
        for (i, param) in method.params.iter().enumerate() {
            if let Some(param_value) = function.get_nth_param(i as u32) {
                self.variables.insert(param.name.clone(), param_value);
            }
        }


        // メソッドボディのコンパイル
        if let Some(body) = &method.body {
            for statement in &body.statements {
                self.compile_statement(statement)?;
            }

            // 戻り値がない場合は void を返す
            if method.return_type.is_none() && !self.builder.get_insert_block().unwrap().get_terminator().is_some() {
                self.builder.build_return(None);
            }
        } else {
            // ボディがない場合はデフォルト値を返す
            if let Some(return_type) = &method.return_type {
                let return_value = self.get_default_value(return_type)?;
                self.builder.build_return(Some(&return_value));
            } else {
                self.builder.build_return(None);
            }
        }

        Ok(function)
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match expr {
            Expression::BinaryOp { left, operator, right } => {
                let left_val = self.compile_expression(left)?;
                let right_val = self.compile_expression(right)?;

                match (left_val, right_val) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = match operator {
                            Operator::Add => self.builder.build_int_add(l, r, "addtmp")
                                .map_err(|e| CodeGenError::Compilation(e.to_string()))?,
                            Operator::Subtract => self.builder.build_int_sub(l, r, "subtmp")
                                .map_err(|e| CodeGenError::Compilation(e.to_string()))?,
                            Operator::Multiply => self.builder.build_int_mul(l, r, "multmp")
                                .map_err(|e| CodeGenError::Compilation(e.to_string()))?,
                            Operator::Divide => self.builder.build_int_signed_div(l, r, "divtmp")
                                .map_err(|e| CodeGenError::Compilation(e.to_string()))?,
                        };
                        Ok(result.as_basic_value_enum())
                    },
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let result = match operator {
                            Operator::Add => self.builder.build_float_add(l, r, "addtmp")
                                .map_err(|e| CodeGenError::Compilation(e.to_string()))?,
                            Operator::Subtract => self.builder.build_float_sub(l, r, "subtmp")
                                .map_err(|e| CodeGenError::Compilation(e.to_string()))?,
                            Operator::Multiply => self.builder.build_float_mul(l, r, "multmp")
                                .map_err(|e| CodeGenError::Compilation(e.to_string()))?,
                            Operator::Divide => self.builder.build_float_div(l, r, "divtmp")
                                .map_err(|e| CodeGenError::Compilation(e.to_string()))?,
                        };
                        Ok(result.as_basic_value_enum())
                    },
                    _ => Err(CodeGenError::InvalidOperation(
                        "Incompatible types for binary operation".to_string()
                    )),
                }
            },
            Expression::Literal(value) => self.compile_literal(value),
            Expression::Variable(name) => {
                self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| CodeGenError::UndefinedVariable(name.clone()))
            },
        }
    }

    fn compile_literal(&self, literal: &LiteralValue) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match literal {
            LiteralValue::Int(value) => {
                Ok(self.context
                    .i32_type()
                    .const_int(*value as u64, false)
                    .as_basic_value_enum())
            },
            LiteralValue::Float(value) => {
                Ok(self.context
                    .f64_type()
                    .const_float(*value)
                    .as_basic_value_enum())
            },
            LiteralValue::Bool(value) => {
                Ok(self.context
                    .bool_type()
                    .const_int(*value as u64, false)
                    .as_basic_value_enum())
            },
            LiteralValue::String(value) => {
                let string_ptr = self.builder.build_global_string_ptr(value, "str")
                    .map_err(|e| CodeGenError::Compilation(e.to_string()))?  // まずResultを処理
                    .as_pointer_value();  // その後ポインタ値を取得
                Ok(BasicValueEnum::PointerValue(string_ptr))
            },
        }
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), CodeGenError> {
        match stmt {
            Statement::Return(expr) => {
                let return_value = self.compile_expression(expr)?;
                self.builder.build_return(Some(&return_value));
            },
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
            },
        }
        Ok(())
    }

    fn convert_type_to_metadata(&self, ty: &Type) -> Result<BasicMetadataTypeEnum<'ctx>, CodeGenError> {
        match ty {
            Type::Int => Ok(self.context.i32_type().into()),
            Type::Float => Ok(self.context.f64_type().into()),
            Type::String => {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                Ok(ptr_type.into())
            },
            Type::Bool => Ok(self.context.bool_type().into()),
            Type::Custom(name) => {
                if let Some(struct_type) = self.struct_types.get(name) {
                    Ok(struct_type.ptr_type(AddressSpace::default()).into())
                } else {
                    Err(CodeGenError::TypeConversion(format!("Unknown custom type: {}", name)))
                }
            },
            _ => Err(CodeGenError::TypeConversion(format!("Unsupported type: {:?}", ty))),
        }
    }


    fn convert_type_to_llvm(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, CodeGenError> {
        match ty {
            Type::Int => Ok(self.context.i32_type().as_basic_type_enum()),
            Type::Float => Ok(self.context.f64_type().as_basic_type_enum()),
            Type::String => {
                Ok(self.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
            },
            Type::Bool => Ok(self.context.bool_type().as_basic_type_enum()),
            Type::Custom(name) => {
                if let Some(struct_type) = self.struct_types.get(name) {
                    Ok(struct_type.ptr_type(AddressSpace::default()).as_basic_type_enum())
                } else {
                    Err(CodeGenError::TypeConversion(format!("Unknown custom type: {}", name)))
                }
            },
            Type::Array(element_type) => {
                let element_type = self.convert_type_to_llvm(element_type)?;
                Ok(self.context.ptr_type(AddressSpace::default()).as_basic_type_enum())
            },
            Type::Optional(inner_type) => {
                self.convert_type_to_llvm(inner_type)
            },
        }
    }

    fn get_default_value(&self, ty: &Type) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match ty {
            Type::Int => Ok(self.context.i32_type().const_zero().as_basic_value_enum()),
            Type::Float => Ok(self.context.f64_type().const_zero().as_basic_value_enum()),
            Type::Bool => Ok(self.context.bool_type().const_zero().as_basic_value_enum()),
            Type::String => {
                let empty_str = self.builder.build_global_string_ptr("", "empty_str")
                    .map_err(|e| CodeGenError::Compilation(e.to_string()))?; // まずResultを処理
                Ok(empty_str.as_basic_value_enum())
            },
            Type::Custom(name) => {
                if let Some(struct_type) = self.struct_types.get(name) {
                    Ok(struct_type.ptr_type(AddressSpace::default())
                        .const_null()
                        .as_basic_value_enum())
                } else {
                    Err(CodeGenError::TypeConversion(format!("Unknown custom type: {}", name)))
                }
            },
            Type::Array(_) => {
                Ok(self.context.ptr_type(AddressSpace::default())
                    .const_null()
                    .as_basic_value_enum())
            },
            Type::Optional(_) => {
                Ok(self.context.i32_type().const_zero().as_basic_value_enum())
            },
        }
    }

    pub fn emit_wasm(&self) -> Result<Vec<u8>, CodeGenError> {
        let triple = TargetTriple::create("wasm32-unknown-unknown");
        self.module.set_triple(&triple);

        let target = Target::from_triple(&triple)
            .map_err(|e| CodeGenError::WasmGen(format!("Failed to create target: {}", e)))?;

        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| CodeGenError::WasmGen("Failed to create target machine".to_string()))?;

        target_machine
            .write_to_memory_buffer(&self.module, FileType::Object)
            .map(|buffer| buffer.as_slice().to_vec())
            .map_err(|e| CodeGenError::WasmGen(format!("Failed to emit WASM: {}", e)))
    }

    pub fn verify_module(&self) -> Result<(), CodeGenError> {
        if self.module.verify().is_err() {
            return Err(CodeGenError::Compilation("Module verification failed".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Actor, ActorType, Method, Field, Type};

    fn create_test_context() -> Context {
        Context::create()
    }

    #[test]
    fn test_basic_method_compilation() {
        let context = create_test_context();
        let mut codegen = CodeGenerator::new(&context, "test");

        let actor = Actor {
            name: "TestActor".to_string(),
            actor_type: ActorType::Single,
            methods: vec![
                Method {
                    name: "test".to_string(),
                    is_async: false,
                    is_sequential: false,
                    is_immediate: false,
                    params: vec![],
                    return_type: Some(Type::Int),
                    body: None,
                }
            ],
            fields: vec![],
        };

        assert!(codegen.compile_actor(&actor).is_ok());
        assert!(codegen.verify_module().is_ok());
    }

    #[test]
    fn test_binary_operation_compilation() {
        let context = create_test_context();
        let mut codegen = CodeGenerator::new(&context, "test");

        // Create a simple add function
        let expression = Expression::BinaryOp {
            left: Box::new(Expression::Literal(LiteralValue::Int(1))),
            operator: Operator::Add,
            right: Box::new(Expression::Literal(LiteralValue::Int(2))),
        };

        let actor = Actor {
            name: "Calculator".to_string(),
            actor_type: ActorType::Single,
            methods: vec![
                Method {
                    name: "add".to_string(),
                    is_async: false,
                    is_sequential: false,
                    is_immediate: false,
                    params: vec![],
                    return_type: Some(Type::Int),
                    body: Some(MethodBody {
                        statements: vec![Statement::Return(expression)],
                    }),
                }
            ],
            fields: vec![],
        };

        assert!(codegen.compile_actor(&actor).is_ok());
        assert!(codegen.verify_module().is_ok());
    }

    #[test]
    fn test_custom_type_compilation() {
        let context = create_test_context();
        let mut codegen = CodeGenerator::new(&context, "test");

        let actor = Actor {
            name: "CustomTypeTest".to_string(),
            actor_type: ActorType::Single,
            methods: vec![],
            fields: vec![
                Field {
                    name: "field".to_string(),
                    field_type: Type::Custom("TestType".to_string()),
                    is_mutable: false,
                    ownership: OwnershipType::Owned,
                }
            ],
        };

        // This should fail because TestType is not defined
        assert!(codegen.compile_actor(&actor).is_err());
    }

    #[test]
    fn test_string_literal_compilation() {
        let context = create_test_context();
        let mut codegen = CodeGenerator::new(&context, "test");

        let expression = Expression::Literal(LiteralValue::String("test".to_string()));
        let actor = Actor {
            name: "StringTest".to_string(),
            actor_type: ActorType::Single,
            methods: vec![
                Method {
                    name: "getString".to_string(),
                    is_async: false,
                    is_sequential: false,
                    is_immediate: false,
                    params: vec![],
                    return_type: Some(Type::String),
                    body: Some(MethodBody {
                        statements: vec![Statement::Return(expression)],
                    }),
                }
            ],
            fields: vec![],
        };

        assert!(codegen.compile_actor(&actor).is_ok());
        assert!(codegen.verify_module().is_ok());
    }
}
