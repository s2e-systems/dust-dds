use generator::rust;
use pest::Parser;

mod generator;
mod parser;
mod preprocessor;

pub fn compile_idl(idl_source: &str) -> Result<String, String> {
    let preprocessed_source = preprocessor::preprocess_input(idl_source)?;

    let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, &preprocessed_source)
        .map_err(|e| format!("Error parsing IDL string: {}", e))?
        .next()
        .expect("Must contain a specification");

    let mut output = String::new();
    rust::generate_rust_source(parsed_idl, &mut output);
    Ok(output)
}
