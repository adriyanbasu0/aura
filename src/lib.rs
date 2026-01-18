pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod typecheck;

use std::fs;
use std::path::Path;

pub fn compile_file(source_path: &str, output_path: Option<&str>) -> Result<(), anyhow::Error> {
    let source = fs::read_to_string(source_path)?;

    let tokens = lexer::lex(&source).map_err(|e| anyhow::anyhow!("Lexing failed: {:?}", e))?;

    let ast = parser::parse(&tokens).map_err(|e| anyhow::anyhow!("Parsing failed: {}", e))?;

    let typed_ast =
        typecheck::typecheck(&ast).map_err(|e| anyhow::anyhow!("Type checking failed: {}", e))?;

    let object = codegen::generate(&typed_ast)
        .map_err(|e| anyhow::anyhow!("Code generation failed: {}", e))?;

    let output = match output_path {
        Some(p) => Path::new(p).to_path_buf(),
        None => Path::new(source_path).with_extension("aura"),
    };

    codegen::write_aura_binary(&object, &output)?;

    println!("Compiled: {} -> {}", source_path, output.display());
    Ok(())
}

pub fn typecheck_file(source_path: &str) -> Result<(), anyhow::Error> {
    let source = fs::read_to_string(source_path)?;
    let tokens = lexer::lex(&source).map_err(|e| anyhow::anyhow!("Lexing failed: {:?}", e))?;
    let ast = parser::parse(&tokens).map_err(|e| anyhow::anyhow!("Parsing failed: {}", e))?;
    let _ =
        typecheck::typecheck(&ast).map_err(|e| anyhow::anyhow!("Type checking failed: {}", e))?;
    Ok(())
}

pub fn dump_binary(binary_path: &str) -> Result<(), anyhow::Error> {
    let data = fs::read(binary_path)?;
    codegen::AuraBinary::dump(&data)?;
    Ok(())
}
