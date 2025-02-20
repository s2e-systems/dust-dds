use std::path::Path;

use generator::rust;
use pest::Parser;

mod generator;
mod parser;
mod preprocessor;

pub fn compile_idl(idl_filepath: &Path) -> Result<String, String> {
    let processed_idl =
        preprocessor::Preprocessor::parse(idl_filepath).map_err(|e| e.to_string())?;
    let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, processed_idl.as_ref())
        .map_err(|e| format!("Error parsing IDL string: {}", e))?
        .next()
        .expect("Must contain a specification");

    let mut output = String::new();
    rust::generate_rust_source(parsed_idl, &mut output);
    Ok(output)
}
