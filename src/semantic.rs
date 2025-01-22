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
}

pub struct SemanticAnalyzer {
    type_environment: HashMap<String, Type>,
    ownership_tracker: HashMap<String, OwnershipType>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            type_environment: HashMap::new(),
            ownership_tracker: HashMap::new(),
        }
    }

    pub fn analyze_actor(&mut self, actor: &Actor) -> Result<(), SemanticError> {
        // Check actor-specific rules
        match actor.actor_type {
            ActorType::Single => self.check_single_actor_constraints(actor)?,
            ActorType::Distributed => self.check_distributed_actor_constraints(actor)?,
        }

        // Analyze fields
        for field in &actor.fields {
            self.analyze_field(field)?;
        }

        // Analyze methods
        for method in &actor.methods {
            self.analyze_method(method, &actor.actor_type)?;
        }

        Ok(())
    }

    fn check_single_actor_constraints(&self, actor: &Actor) -> Result<(), SemanticError> {
        // Verify single actor doesn't use distributed features
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
        // Verify distributed actor follows distributed rules
        for field in &actor.fields {
            if matches!(field.ownership, OwnershipType::Shared) {
                self.verify_shared_field_constraints(field)?;
            }
        }

        Ok(())
    }

    fn analyze_field(&mut self, field: &Field) -> Result<(), SemanticError> {
        // Register field type
        self.type_environment
            .insert(field.name.clone(), field.field_type.clone());

        // Check ownership rules
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

    fn analyze_method(
        &mut self,
        method: &Method,
        actor_type: &ActorType,
    ) -> Result<(), SemanticError> {
        // Check async/sequential constraints
        if method.is_sequential && !method.is_async {
            return Err(SemanticError::AsyncError(
                "Sequential methods must be async".to_string(),
            ));
        }

        // Check immediate init constraints
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

        // Analyze method parameters and return type
        for param in &method.params {
            self.verify_parameter_type(param)?;
        }

        if let Some(return_type) = &method.return_type {
            self.verify_return_type(return_type)?;
        }

        Ok(())
    }

    fn verify_parameter_type(&self, param: &Parameter) -> Result<(), SemanticError> {
        // Add parameter type verification logic
        Ok(())
    }

    fn verify_return_type(&self, return_type: &Type) -> Result<(), SemanticError> {
        // Add return type verification logic
        Ok(())
    }

    fn verify_shared_field_constraints(&self, field: &Field) -> Result<(), SemanticError> {
        // Add shared field constraints verification
        Ok(())
    }
}
