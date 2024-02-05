use generator::rust;
use pest::Parser;

mod generator;
mod parser;

pub fn compile_idl(idl_source: &str) -> Result<String, String> {
    let preprocessed_source = preprocess_input(idl_source)?;

    let parsed_idl = parser::IdlParser::parse(parser::Rule::specification, &preprocessed_source)
        .map_err(|e| format!("Error parsing IDL string: {}", e))?
        .next()
        .expect("Must contain a specification");

    let mut output = String::new();
    rust::generate_rust_source(parsed_idl, &mut output);
    Ok(output)
}

fn preprocess_input(idl_source: &str) -> Result<String, String> {
    for line in idl_source.lines() {
        let mut split_line = line.split(&[' ', '\t']).peekable();

        if let Some(&"#define") = split_line.peek() {
            todo!("#define not yet implemented")
        } else {
        }
    }
    Ok(idl_source.to_string())
}
