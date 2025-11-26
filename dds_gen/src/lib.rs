mod generator;
mod parser;
mod preprocessor;

use self::generator::{Context, rust};
use pest::Parser;
use std::path::Path;

pub fn compile_idl(idl_filepath: &Path) -> Result<String, String> {
    let processed_idl =
        preprocessor::Preprocessor::parse(idl_filepath).map_err(|e| e.to_string())?;
    let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, processed_idl.as_ref())
        .map_err(|e| format!("Error parsing IDL string: {e}"))?
        .next()
        .expect("Must contain a specification");

    let mut ctx = Context::<String>::default();
    rust::generate_rust_source(parsed_idl, &mut ctx).map_err(|err| err.to_string())?;

    Ok(ctx.writer)
}
