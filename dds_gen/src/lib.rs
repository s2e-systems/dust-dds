use generator::rust;
use pest::Parser;

mod generator;
mod parser;

pub fn compile_idl(idl_source: &str) -> Result<String, String> {
    let mut idl_parser = match parser::IdlParser::parse(parser::Rule::specification, idl_source) {
        Ok(parsed_idl) => parsed_idl,
        Err(e) => panic!("Error parsing IDL string: {}", e),
    };

    let parsed_idl = match idl_parser.next() {
        Some(parsed_idl) => parsed_idl,
        None => panic!("Must contain a specification"),
    };



    // let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, idl_source)
    //     .map_err(|e| format!("Error parsing IDL string: {}", e))?
    //     .next()
    //     .expect("Must contain a specification");

    let mut output = String::new();
    rust::generate_rust_source(parsed_idl, &mut output);
    Ok(output)
}
