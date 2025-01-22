use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use std::collections::HashMap;

mod lexer;
mod parser;
mod ast;
mod semantic;
mod codegen;
mod ownership;

fn main() {
    println!("Replica Compiler");
    let context = Context::create();
    let module = context.create_module("replica_module");
    let builder = context.create_builder();

    // Setup basic compiler pipeline
    // TODO: Implement compilation pipeline
}
