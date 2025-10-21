use std::path::Path;

//use generator::rust;
//use pest::Parser;

pub use crate::parser::parse_idl;
pub use crate::generator::rust::generate_rust_code;
use crate::parser::Rule;

mod generator;
mod parser;
mod preprocessor;

pub fn compile_idl(idl_filepath: &Path) -> Result<String, String> {
    let processed_idl =
        preprocessor::Preprocessor::parse(idl_filepath).map_err(|e| e.to_string())?;
    let parsed_idl = parser::parse_idl(&processed_idl, Rule::specification)
        .map_err(|e| format!("Error parsing IDL string: {}", e))?;
    /*let parsed_idl = parse_idl(parser::Rule::specification, processed_idl.as_ref())
        .map_err(|e| format!("Error parsing IDL string: {e}"))?
        .next()
        .expect("Must contain a specification");*/

    let mut output = String::new();
    let _ = generate_rust_code(Ok(parsed_idl), &mut output);
    /*for s in &parsed_idl {
        output.push_str(&rust::generate_rust_def(s));
    }*/
    Ok(output)
}