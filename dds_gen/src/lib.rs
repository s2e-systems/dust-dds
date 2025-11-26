mod generator;
mod parser;
mod preprocessor;

use self::generator::{Generator, RustGenerator};
use pest::Parser;
use std::path::Path;

pub fn compile_idl(idl_filepath: &Path) -> Result<String, String> {
    let processed_idl =
        preprocessor::Preprocessor::parse(idl_filepath).map_err(|e| e.to_string())?;
    let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, processed_idl.as_ref())
        .map_err(|e| format!("Error parsing IDL string: {e}"))?
        .next()
        .expect("Must contain a specification");

    let mut rust_generator = RustGenerator::default();
    rust_generator.generate(parsed_idl);
    Ok(rust_generator.into_writer())
}
