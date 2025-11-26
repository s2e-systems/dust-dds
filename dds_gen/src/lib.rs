mod generator;
mod parser;
mod preprocessor;

use pest::Parser;
use std::path::Path;

use crate::generator::rust::RustGenerator;

pub fn compile_idl(idl_filepath: &Path) -> Result<String, String> {
    let processed_idl =
        preprocessor::Preprocessor::parse(idl_filepath).map_err(|e| e.to_string())?;
    let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, processed_idl.as_ref())
        .map_err(|e| format!("Error parsing IDL string: {e}"))?
        .next()
        .expect("Must contain a specification");

    let mut output = String::new();
    let mut rust_generator = RustGenerator::new(&mut output);
    rust_generator.generate_source(parsed_idl);
    Ok(output)
}
