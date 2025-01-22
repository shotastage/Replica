use inkwell::{
    context::Context,
    module::Module,
    builder::Builder,
    values::{FunctionValue, BasicValueEnum},
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
use crate::ast::{Actor, Method, Type};
use std::collections::HashMap;
use inkwell::types::BasicType;
use inkwell::values::BasicValue;

pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    actor_methods: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        // Initialize WASM target
        Target::initialize_webassembly(&InitializationConfig::default());

        CodeGenerator {
            context,
            module,
            builder,
            actor_methods: HashMap::new(),
        }
    }

    pub fn compile_actor(&mut self, actor: &Actor) -> Result<(), String> {
        // Create actor struct type
        let _struct_type = self.context.opaque_struct_type(&actor.name);

        // Define methods
        for method in &actor.methods {
            let function = self.compile_method(method)?;
            self.actor_methods.insert(method.name.clone(), function);
        }

        Ok(())
    }

    fn compile_method(
        &mut self,
        method: &Method,
    ) -> Result<FunctionValue<'ctx>, String> {
        // Convert parameter types to LLVM metadata types directly
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = method.params
            .iter()
            .map(|param| self.convert_type_to_metadata(&param.param_type))
            .collect::<Result<Vec<_>, _>>()?;

        // Create function type
        let fn_type = match &method.return_type {
            Some(ty) => {
                let return_type = self.convert_type_to_llvm(ty)?;
                return_type.fn_type(&param_types, false)
            },
            None => self.context.void_type().fn_type(&param_types, false),
        };

        let function = self.module.add_function(&method.name, fn_type, None);

        // Create entry basic block
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        // Add method implementation here
        // For now, just return a default value
        match &method.return_type {
            Some(ty) => {
                let return_value = self.get_default_value(ty)?;
                self.builder.build_return(Some(&return_value));
            }
            None => {
                self.builder.build_return(None);
            }
        }

        Ok(function)
    }

    fn convert_type_to_metadata(&self, ty: &Type) -> Result<BasicMetadataTypeEnum<'ctx>, String> {
        match ty {
            Type::Int => Ok(self.context.i32_type().into()),
            Type::Float => Ok(self.context.f64_type().into()),
            Type::String => {
                // Use AddressSpace::default() for the default address space
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                Ok(ptr_type.into())
            },
            Type::Bool => Ok(self.context.bool_type().into()),
            _ => Err(format!("Unsupported type: {:?}", ty))
        }
    }

    fn convert_type_to_llvm(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            Type::Int => Ok(self.context.i32_type().as_basic_type_enum()),
            Type::Float => Ok(self.context.f64_type().as_basic_type_enum()),
            Type::String => {
                // Use AddressSpace::default() for the default address space
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                Ok(ptr_type.as_basic_type_enum())
            },
            Type::Bool => Ok(self.context.bool_type().as_basic_type_enum()),
            _ => Err(format!("Unsupported type: {:?}", ty))
        }
    }

    fn get_default_value(&self, ty: &Type) -> Result<BasicValueEnum<'ctx>, String> {
        match ty {
            Type::Int => Ok(self.context.i32_type().const_zero().as_basic_value_enum()),
            Type::Float => Ok(self.context.f64_type().const_zero().as_basic_value_enum()),
            Type::Bool => Ok(self.context.bool_type().const_zero().as_basic_value_enum()),
            _ => Err(format!("Default value not implemented for type: {:?}", ty))
        }
    }

    pub fn emit_wasm(&self) -> Result<Vec<u8>, String> {
        let triple = TargetTriple::create("wasm32-unknown-unknown");
        self.module.set_triple(&triple);

        let target = Target::from_triple(&triple)
            .map_err(|e| format!("Failed to create target: {}", e))?;

        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| "Failed to create target machine".to_string())?;

        target_machine
            .write_to_memory_buffer(&self.module, FileType::Object)
            .map(|buffer| buffer.as_slice().to_vec())
            .map_err(|e| format!("Failed to emit WASM: {}", e))
    }
}
