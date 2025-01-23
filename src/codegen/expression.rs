use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValue, BasicValueEnum},
    FloatPredicate, IntPredicate,
};
use std::collections::HashMap;

use super::{
    error::{CodeGenError, CodeGenResult},
    type_converter::TypeConverter,
};
use crate::ast::{Expression, LiteralValue, Operator};

/// Compiles Replica expressions to LLVM IR
pub struct ExpressionCompiler<'ctx> {
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    type_converter: TypeConverter<'ctx>,
    variables: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> ExpressionCompiler<'ctx> {
    /// Creates a new ExpressionCompiler instance
    pub fn new(context: &'ctx Context, builder: &'ctx Builder<'ctx>) -> Self {
        ExpressionCompiler {
            context,
            builder,
            type_converter: TypeConverter::new(context),
            variables: HashMap::new(),
        }
    }

    /// Registers a variable in the current scope
    pub fn register_variable(&mut self, name: String, value: BasicValueEnum<'ctx>) {
        self.variables.insert(name, value);
    }

    /// Clears all registered variables
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// Compiles an expression to LLVM IR
    pub fn compile_expression(&self, expr: &Expression) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match expr {
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => self.compile_binary_operation(left, operator, right),
            Expression::Literal(value) => self.compile_literal(value),
            Expression::Variable(name) => self.compile_variable(name),
        }
    }

    /// Compiles a binary operation
    fn compile_binary_operation(
        &self,
        left: &Expression,
        operator: &Operator,
        right: &Expression,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        let left_value = self.compile_expression(left)?;
        let right_value = self.compile_expression(right)?;

        match (left_value, right_value) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                let result = match operator {
                    Operator::Add => self
                        .builder
                        .build_int_add(l, r, "addtmp")
                        .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?,
                    Operator::Subtract => self
                        .builder
                        .build_int_sub(l, r, "subtmp")
                        .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?,
                    Operator::Multiply => self
                        .builder
                        .build_int_mul(l, r, "multmp")
                        .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?,
                    Operator::Divide => self
                        .builder
                        .build_int_signed_div(l, r, "divtmp")
                        .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?,
                };
                Ok(result.as_basic_value_enum())
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                let result = match operator {
                    Operator::Add => self
                        .builder
                        .build_float_add(l, r, "addtmp")
                        .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?,
                    Operator::Subtract => self
                        .builder
                        .build_float_sub(l, r, "subtmp")
                        .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?,
                    Operator::Multiply => self
                        .builder
                        .build_float_mul(l, r, "multmp")
                        .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?,
                    Operator::Divide => self
                        .builder
                        .build_float_div(l, r, "divtmp")
                        .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?,
                };
                Ok(result.as_basic_value_enum())
            }
            _ => Err(CodeGenError::ExpressionCompilation(
                "Incompatible types for binary operation".to_string(),
            )),
        }
    }

    /// Compiles a literal value
    fn compile_literal(&self, value: &LiteralValue) -> CodeGenResult<BasicValueEnum<'ctx>> {
        match value {
            LiteralValue::Int(i) => Ok(self
                .context
                .i32_type()
                .const_int(*i as u64, false)
                .as_basic_value_enum()),
            LiteralValue::Float(f) => Ok(self
                .context
                .f64_type()
                .const_float(*f)
                .as_basic_value_enum()),
            LiteralValue::String(s) => {
                let string_ptr = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?;
                Ok(string_ptr.as_basic_value_enum())
            }
            LiteralValue::Bool(b) => Ok(self
                .context
                .bool_type()
                .const_int(*b as u64, false)
                .as_basic_value_enum()),
        }
    }

    /// Compiles a variable reference
    fn compile_variable(&self, name: &str) -> CodeGenResult<BasicValueEnum<'ctx>> {
        self.variables
            .get(name)
            .cloned()
            .ok_or_else(|| CodeGenError::UndefinedVariable(name.to_string()))
    }

    /// Compiles a comparison operation
    pub fn compile_comparison(
        &self,
        left: &Expression,
        predicate: IntPredicate,
        right: &Expression,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        let left_value = self.compile_expression(left)?;
        let right_value = self.compile_expression(right)?;

        match (left_value, right_value) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                let result = self
                    .builder
                    .build_int_compare(predicate, l, r, "cmptmp")
                    .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?;
                Ok(result.as_basic_value_enum())
            }
            _ => Err(CodeGenError::ExpressionCompilation(
                "Invalid types for comparison".to_string(),
            )),
        }
    }

    /// Compiles a floating point comparison operation
    pub fn compile_float_comparison(
        &self,
        left: &Expression,
        predicate: FloatPredicate,
        right: &Expression,
    ) -> CodeGenResult<BasicValueEnum<'ctx>> {
        let left_value = self.compile_expression(left)?;
        let right_value = self.compile_expression(right)?;

        match (left_value, right_value) {
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                let result = self
                    .builder
                    .build_float_compare(predicate, l, r, "cmptmp")
                    .map_err(|e| CodeGenError::ExpressionCompilation(e.to_string()))?;
                Ok(result.as_basic_value_enum())
            }
            _ => Err(CodeGenError::ExpressionCompilation(
                "Invalid types for float comparison".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;
    use inkwell::FloatPredicate;
    use inkwell::IntPredicate;

    fn create_test_compiler<'ctx>(
        context: &'ctx Context,
        builder: &'ctx Builder<'ctx>,
    ) -> ExpressionCompiler<'ctx> {
        ExpressionCompiler::new(context, builder)
    }

    #[test]
    fn test_literal_compilation() {
        let context = Context::create();
        let builder = context.create_builder();
        let compiler = create_test_compiler(&context, &builder);

        let int_literal = LiteralValue::Int(42);
        let float_literal = LiteralValue::Float(3.14);
        let string_literal = LiteralValue::String("test".to_string());
        let bool_literal = LiteralValue::Bool(true);

        assert!(compiler.compile_literal(&int_literal).is_ok());
        assert!(compiler.compile_literal(&float_literal).is_ok());
        assert!(compiler.compile_literal(&string_literal).is_ok());
        assert!(compiler.compile_literal(&bool_literal).is_ok());
    }

    #[test]
    fn test_binary_operation() {
        let context = Context::create();
        let builder = context.create_builder();
        let module = context.create_module("test");

        // 関数を作成してその中でテストを実行
        let fn_type = context.void_type().fn_type(&[], false);
        let function = module.add_function("test", fn_type, None);
        let basic_block = context.append_basic_block(function, "entry");
        builder.position_at_end(basic_block);

        let compiler = create_test_compiler(&context, &builder);

        let left = Expression::Literal(LiteralValue::Int(10));
        let right = Expression::Literal(LiteralValue::Int(5));
        let add_op = Operator::Add;

        let result = compiler.compile_binary_operation(&left, &add_op, &right);
        assert!(result.is_ok());
    }

    #[test]
    fn test_variable_compilation() {
        let context = Context::create();
        let builder = context.create_builder();
        let mut compiler = create_test_compiler(&context, &builder);

        // 変数を登録
        let value = context
            .i32_type()
            .const_int(42, false)
            .as_basic_value_enum();
        compiler.register_variable("test_var".to_string(), value);

        // 変数の参照をコンパイル
        let result = compiler.compile_variable("test_var");
        assert!(result.is_ok());

        // 未定義の変数
        let result = compiler.compile_variable("undefined_var");
        assert!(result.is_err());
    }
}
