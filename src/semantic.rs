use crate::ast::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Ownership error: {0}")]
    OwnershipError(String),
    #[error("Invalid actor operation: {0}")]
    InvalidActorOperation(String),
    #[error("Async/await error: {0}")]
    AsyncError(String),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub struct SemanticAnalyzer {
    type_environment: HashMap<String, Type>,
    ownership_tracker: HashMap<String, OwnershipType>,
    current_scope: Vec<HashMap<String, Type>>, // スコープスタック
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            type_environment: HashMap::new(),
            ownership_tracker: HashMap::new(),
            current_scope: vec![HashMap::new()],
        }
    }

    pub fn analyze_actor(&mut self, actor: &Actor) -> Result<(), SemanticError> {
        // アクター固有のルールをチェック
        match actor.actor_type {
            ActorType::Single => self.check_single_actor_constraints(actor)?,
            ActorType::Distributed => self.check_distributed_actor_constraints(actor)?,
        }

        // フィールドの解析
        for field in &actor.fields {
            self.analyze_field(field)?;
        }

        // メソッドの解析
        for method in &actor.methods {
            self.analyze_method(method, &actor.actor_type)?;
        }

        Ok(())
    }

    fn check_single_actor_constraints(&self, actor: &Actor) -> Result<(), SemanticError> {
        // 分散機能を使用していないことを確認
        for method in &actor.methods {
            if method.is_async && !method.is_immediate {
                return Err(SemanticError::InvalidActorOperation(
                    "Single actor cannot have async methods except immediate init".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn check_distributed_actor_constraints(&self, actor: &Actor) -> Result<(), SemanticError> {
        // distributed actorのルールに従っているか確認
        for field in &actor.fields {
            if matches!(field.ownership, OwnershipType::Shared) {
                self.verify_shared_field_constraints(field)?;
            }
        }

        Ok(())
    }

    fn analyze_field(&mut self, field: &Field) -> Result<(), SemanticError> {
        // フィールドの型を登録
        self.type_environment
            .insert(field.name.clone(), field.field_type.clone());

        // 所有権ルールのチェック
        match field.ownership {
            OwnershipType::Moved => {
                if field.is_mutable {
                    return Err(SemanticError::OwnershipError(
                        "Moved fields cannot be mutable".to_string(),
                    ));
                }
            }
            OwnershipType::Shared => {
                if !field.is_mutable {
                    return Err(SemanticError::OwnershipError(
                        "Shared fields must be mutable".to_string(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn analyze_expression(&self, expr: &Expression) -> Result<Type, SemanticError> {
        match expr {
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                match operator {
                    Operator::Add | Operator::Subtract | Operator::Multiply | Operator::Divide => {
                        // 数値演算の型チェック
                        match (&left_type, &right_type) {
                            (Type::Int, Type::Int) => Ok(Type::Int),
                            (Type::Float, Type::Float) => Ok(Type::Float),
                            _ => Err(SemanticError::TypeError(format!(
                                "Invalid operand types for arithmetic operation: {:?} and {:?}",
                                left_type, right_type
                            ))),
                        }
                    }
                }
            }
            Expression::Literal(value) => match value {
                LiteralValue::Int(_) => Ok(Type::Int),
                LiteralValue::Float(_) => Ok(Type::Float),
                LiteralValue::String(_) => Ok(Type::String),
                LiteralValue::Bool(_) => Ok(Type::Bool),
            },
            Expression::Variable(name) => {
                // 変数の型を現在のスコープから探す
                for scope in self.current_scope.iter().rev() {
                    if let Some(var_type) = scope.get(name) {
                        return Ok(var_type.clone());
                    }
                }
                Err(SemanticError::UndefinedVariable(name.clone()))
            }
        }
    }

    fn analyze_statement(
        &mut self,
        stmt: &Statement,
        expected_return_type: &Option<Type>,
    ) -> Result<(), SemanticError> {
        match stmt {
            Statement::Return(expr) => {
                let expr_type = self.analyze_expression(expr)?;
                if let Some(expected) = expected_return_type {
                    if !self.check_type_compatibility(expected, &expr_type) {
                        return Err(SemanticError::TypeError(format!(
                            "Return type mismatch: expected {:?}, found {:?}",
                            expected, expr_type
                        )));
                    }
                }
                Ok(())
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
                Ok(())
            }
        }
    }

    fn analyze_method(
        &mut self,
        method: &Method,
        actor_type: &ActorType,
    ) -> Result<(), SemanticError> {
        // 新しいスコープを作成
        self.current_scope.push(HashMap::new());

        // パラメータをスコープに追加
        for param in &method.params {
            self.current_scope
                .last_mut()
                .unwrap()
                .insert(param.name.clone(), param.param_type.clone());
        }

        // async/sequentialのチェック
        if method.is_sequential && !method.is_async {
            return Err(SemanticError::AsyncError(
                "Sequential methods must be async".to_string(),
            ));
        }

        // immediateイニシャライザのチェック
        if method.is_immediate {
            if method.name != "init" {
                return Err(SemanticError::AsyncError(
                    "Only init method can be immediate".to_string(),
                ));
            }

            if matches!(actor_type, ActorType::Distributed) {
                return Err(SemanticError::AsyncError(
                    "Distributed actors cannot have immediate init".to_string(),
                ));
            }
        }

        // メソッドボディの解析
        if let Some(body) = &method.body {
            for statement in &body.statements {
                self.analyze_statement(statement, &method.return_type)?;
            }
        }

        // スコープを削除
        self.current_scope.pop();

        // パラメータと戻り値の型の検証
        for param in &method.params {
            self.verify_parameter_type(param)?;
        }

        if let Some(return_type) = &method.return_type {
            self.verify_return_type(return_type)?;
        }

        Ok(())
    }

    fn verify_parameter_type(&self, param: &Parameter) -> Result<(), SemanticError> {
        // パラメータの型が有効かチェック
        match &param.param_type {
            Type::Custom(name) => {
                if !self.type_environment.contains_key(name) {
                    return Err(SemanticError::TypeError(format!(
                        "Unknown type {} for parameter {}",
                        name, param.name
                    )));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn verify_return_type(&self, return_type: &Type) -> Result<(), SemanticError> {
        // 戻り値の型が有効かチェック
        match return_type {
            Type::Custom(name) => {
                if !self.type_environment.contains_key(name) {
                    return Err(SemanticError::TypeError(format!(
                        "Unknown return type {}",
                        name
                    )));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn verify_shared_field_constraints(&self, field: &Field) -> Result<(), SemanticError> {
        // 共有フィールドの制約をチェック
        match &field.field_type {
            Type::Custom(_) => {
                // カスタム型の共有フィールドに対する追加チェック
                if !field.is_mutable {
                    return Err(SemanticError::OwnershipError(
                        "Shared fields of custom type must be mutable".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn check_type_compatibility(&self, expected: &Type, found: &Type) -> bool {
        match (expected, found) {
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::String, Type::String) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Custom(e), Type::Custom(f)) => e == f,
            (Type::Array(e), Type::Array(f)) => self.check_type_compatibility(e, f),
            (Type::Optional(e), Type::Optional(f)) => self.check_type_compatibility(e, f),
            (Type::Optional(e), f) => self.check_type_compatibility(e, f),
            _ => false,
        }
    }
}

// テストモジュール
#[cfg(test)]
mod tests {
    use super::*;

    // 基本的な型チェックのテスト
    #[test]
    fn test_basic_type_checking() {
        let analyzer = SemanticAnalyzer::new();
        assert!(analyzer.check_type_compatibility(&Type::Int, &Type::Int));
        assert!(!analyzer.check_type_compatibility(&Type::Int, &Type::Float));
    }

    // 配列型の互換性テスト
    #[test]
    fn test_array_type_compatibility() {
        let analyzer = SemanticAnalyzer::new();
        assert!(analyzer.check_type_compatibility(
            &Type::Array(Box::new(Type::Int)),
            &Type::Array(Box::new(Type::Int))
        ));
        assert!(!analyzer.check_type_compatibility(
            &Type::Array(Box::new(Type::Int)),
            &Type::Array(Box::new(Type::Float))
        ));
    }

    // オプショナル型のテスト
    #[test]
    fn test_optional_type_compatibility() {
        let analyzer = SemanticAnalyzer::new();
        assert!(analyzer.check_type_compatibility(&Type::Optional(Box::new(Type::Int)), &Type::Int));
        assert!(analyzer.check_type_compatibility(
            &Type::Optional(Box::new(Type::Int)),
            &Type::Optional(Box::new(Type::Int))
        ));
    }
}
