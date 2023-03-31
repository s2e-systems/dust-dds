use pest::Parser;
use syntax::Analyser;

mod idl;
mod mappings;
mod parser;
mod syntax;

pub fn compile_idl(idl_source: &str) -> Result<String, ()> {
    let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, &idl_source).unwrap();
    let idl_spec = syntax::specification().analyse(parsed_idl.into()).unwrap();

    Ok(idl_spec
        .value
        .into_iter()
        .flat_map(mappings::rust::definition)
        .collect())
}
