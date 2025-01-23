//! Code generation module for compiling Replica actors to WASM.
//! This module handles the transformation of AST to LLVM IR and final WASM output.

mod error;
mod expression;
mod generator;
mod type_converter;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::OptimizationLevel;

pub use error::{CodeGenError, CodeGenResult};
pub use generator::CodeGenerator;

// Re-export only the necessary types and traits
pub use expression::ExpressionCompiler;
pub use type_converter::TypeConverter;

pub struct ContextWrapper {
    context: Context,
}

/// Configuration options for code generation
#[derive(Debug, Clone)]
pub struct CodeGenOptions {
    /// Optimization level for LLVM
    pub optimization_level: OptimizationLevel,
    /// Whether to enable debug information
    pub debug_mode: bool,
    /// Target triple for WASM compilation
    pub target_triple: String,
}

impl Default for CodeGenOptions {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Default,
            debug_mode: false,
            target_triple: String::from("wasm32-unknown-unknown"),
        }
    }
}

/// Creates a new code generator with the given context and module name
pub fn create_generator<'ctx>(
    context: &'ctx Context,
    module_name: &str,
    options: Option<CodeGenOptions>,
) -> CodeGenResult<CodeGenerator<'ctx>> {
    CodeGenerator::new(context, module_name, options.unwrap_or_default())
}

/// Utility function to create a new context and code generator in one step
pub fn create_generator_with_context(
    module_name: &str,
    options: Option<CodeGenOptions>,
) -> CodeGenResult<(ContextWrapper, CodeGenerator)> {
    let context_wrapper = ContextWrapper {
        context: Context::create(),
    };
    let generator = create_generator(&context_wrapper.context, module_name, options)?;
    Ok((context_wrapper, generator))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Actor, ActorType};

    #[test]
    fn test_create_generator() {
        let context = Context::create();
        let result = create_generator(&context, "test_module", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_generator_with_context() {
        let result = create_generator_with_context("test_module", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_generator_with_options() {
        let context = Context::create();
        let options = CodeGenOptions {
            optimization_level: OptimizationLevel::Aggressive,
            debug_mode: true,
            target_triple: String::from("wasm32-unknown-unknown"),
        };

        let result = create_generator(&context, "test_module", Some(options));
        assert!(result.is_ok());
    }

    #[test]
    fn test_generator_compilation() {
        let (context, mut generator) =
            create_generator_with_context("test_module", None).expect("Failed to create generator");

        let test_actor = Actor {
            name: String::from("TestActor"),
            actor_type: ActorType::Single,
            methods: vec![],
            fields: vec![],
        };

        let result = generator.compile_actor(&test_actor);
        assert!(result.is_ok());
    }
}
