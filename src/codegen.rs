use inkwell::{
    context::Context,
    module::Module,
    builder::Builder,
    values::{FunctionValue, BasicValueEnum},
    types::BasicTypeEnum,
    targets::{Target, TargetMachine, InitializationConfig},
};
use crate::ast::{Actor, Method, Type};
use std::collections::HashMap;

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
        Target::initialize_webassembly(&InitializationConfig::default())
            .expect("Failed to initialize WebAssembly target");

        CodeGenerator {
            context,
            module,
            builder,
            actor_methods: HashMap::new(),
        }
    }

    pub fn compile_actor(&mut self, actor: &Actor) -> Result<(), String> {
        // Create actor struct type
        let struct_type = self.context.opaque_struct_type(&actor.name);

        // Define methods
        for method in &actor.methods {
            let function = self.compile_method(method, &struct_type)?;
            self.actor_methods.insert(method.name.clone(), function);
        }

        Ok(())
    }

    fn compile_method(&mut self, method: &Method, actor_type: &BasicTypeEnum<'ctx>) -> Result<FunctionValue<'ctx>, String> {
        // Convert parameter types to LLVM types
        let param_types: Vec<BasicTypeEnum> = method.params
            .iter()
            .map(|param| self.convert_type_to_llvm(&param.param_type))
            .collect::<Result<Vec<_>, _>>()?;

        // Create function type
        let return_type = match &method.return_type {
            Some(ty) => self.convert_type_to_llvm(ty)?,
            None => self.context.void_type().as_basic_type_enum(),
        };

        let fn_type = return_type.fn_type(&param_types, false);
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

    fn convert_type_to_llvm(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            Type::Int => Ok(self.context.i32_type().as_basic_type_enum()),
            Type::Float => Ok(self.context.f64_type().as_basic_type_enum()),
            Type::String => Ok(self.context.i8_type().ptr_type(inkwell::AddressSpace::Generic).as_basic_type_enum()),
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
        let target_triple = Target::get_first().unwrap().get_triple();
        let target_machine = Target::get_first()
            .unwrap()
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                inkwell::OptimizationLevel::Default,
                inkwell::RelocationModel::Default,
                inkwell::CodeModel::Default,
            )
            .unwrap();

        target_machine
            .write_to_memory_buffer(&self.module, inkwell::targets::FileType::Object)
            .map(|buffer| buffer.as_slice().to_vec())
            .map_err(|e| format!("Failed to emit WASM: {}", e))
    }
}
