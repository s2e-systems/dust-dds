use std::path::Path;

use generator::rust;
//use pest::Parser;

pub use crate::parser::parse_idl;
pub use crate::generator::rust::generate_rust_def;

mod generator;
mod parser;
mod preprocessor;

pub fn compile_idl(idl_filepath: &Path) -> Result<String, String> {
    let processed_idl = preprocessor::Preprocessor::parse(idl_filepath)
        .map_err(|e| e.to_string())?;

        println!("IDL INPUT:\n{}", processed_idl);
    /*let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, processed_idl.as_ref())
        .map_err(|e| format!("Error parsing IDL string: {}", e))?
        .next()
        .expect("Must contain a specification");*/
    let parsed_idl = parser::parse_idl(&processed_idl)
        .map_err(|e| format!("Error parsing IDL string: {}", e))?;

    let mut output = String::new();
    for s in &parsed_idl {
        output.push_str(&rust::generate_rust_def(s));
    }
    Ok(output)
}
