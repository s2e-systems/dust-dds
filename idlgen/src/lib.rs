use pest::Parser;
use syntax::Analyser;

mod idl;
mod mappings;
mod parser;
mod syntax;

pub fn compile_idl(idl_source: &str) -> Result<String, String> {
    let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, &idl_source)
        .map_err(|e| format!("Error parsing IDL string: {:?}", e))?;
    let idl_spec = syntax::specification()
        .analyse(parsed_idl.into())
        .map_err(|e| format!("Error compiling IDL: {:?}", e.pretty_print()))?;

    Ok(idl_spec
        .value
        .into_iter()
        .flat_map(mappings::rust::definition)
        .collect())
}
