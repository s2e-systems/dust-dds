use std::{fs::File, io::Write};

use dust_idlgen::{
    idl_parser,
    idl_syntax::{self, Analyser},
    rust_mapping,
};
use pest::Parser;

fn main() {
    let idl_path = "HelloWorld.idl";
    let idl_src = std::fs::read_to_string(idl_path).expect("(;_;) Couldn't read IDL source file!");

    let result = idl_parser::IdlParser::parse(idl_parser::Rule::specification, &idl_src)
        .expect("Couldn't parse IDL file");

    let mut file = File::create("hello_world.rs").expect("Failed to create file");

    let spec = idl_syntax::specification()
        .analyse(result.tokens())
        .expect("Couldn't analyse IDL syntax");

    for def in spec.value {
        for line in rust_mapping::definition(def) {
            file.write(line.as_bytes())
                .expect("Failed to write to file");
        }
    }
}
