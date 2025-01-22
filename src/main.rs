use inkwell::context::Context;
use std::fs;
use std::path::Path;
use std::process;

mod lexer;
mod parser;
mod ast;
mod semantic;
mod codegen;
mod ownership;

use crate::semantic::SemanticAnalyzer;

fn compile_file(source_path: &Path) -> Result<Vec<u8>, String> {
    // Read source file
    let source = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read source file: {}", e))?;

    // Lexical analysis
    let (_, tokens) = lexer::lex(&source)
        .map_err(|e| format!("Lexer error: {}", e))?;

    // Parsing
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse_actor()
        .map_err(|e| format!("Parser error: {}", e))?;

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_actor(&ast)
        .map_err(|e| format!("Semantic analysis error: {}", e))?;

    // Code generation
    let context = Context::create();
    let module_name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");

    let mut code_gen = codegen::CodeGenerator::new(&context, module_name);
    code_gen.compile_actor(&ast)
        .map_err(|e| format!("Code generation error: {}", e))?;

    // Emit WASM
    code_gen.emit_wasm()
        .map_err(|e| format!("WASM emission error: {}", e))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    println!("Compiling {} to {}", input_path.display(), output_path.display());

    // Compile the source file
    match compile_file(input_path) {
        Ok(wasm_bytes) => {
            // Write the output file
            if let Err(e) = fs::write(output_path, wasm_bytes) {
                eprintln!("Failed to write output file: {}", e);
                process::exit(1);
            }
            println!("Successfully compiled to WASM");
        }
        Err(e) => {
            eprintln!("Compilation error: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_basic_compilation() {
        let test_source = r#"
            actor TestActor {
                var value: Int

                func getValue() -> Int {
                    return value
                }
            }
        "#;

        let test_path = PathBuf::from("test.replica");
        fs::write(&test_path, test_source).unwrap();

        let result = compile_file(&test_path);
        fs::remove_file(&test_path).unwrap();

        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }
}
