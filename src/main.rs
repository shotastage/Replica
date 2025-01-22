// src/main.rs
use inkwell::context::Context;

mod ast;
mod codegen;
mod lexer;
mod ownership;
mod parser;
mod semantic;

fn main() {
    println!("Replica Compiler");
}
