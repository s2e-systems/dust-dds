use std::{fs::File, io::Write};

use dust_idlgen::{
    parser,
    syntax::{self, Analyser},
    mappings::rust,
};
use pest::Parser;

fn main() {
    let idl_path = "HelloWorld.idl";
    let idl_src = std::fs::read_to_string(idl_path).expect("(;_;) Couldn't read IDL source file!");

    let result = parser::IdlParser::parse(parser::Rule::specification, &idl_src)
        .expect("Couldn't parse IDL file");

    let mut file = File::create("hello_world.rs").expect("Failed to create file");

    let spec = syntax::specification()
        .analyse(result.into())
        .expect("Couldn't analyse IDL syntax");

    for def in spec.value {
        for line in rust::definition(def) {
            file.write_all(line.as_bytes())
                .expect("Failed to write to file");
        }
    }


    let idl_path = "BigData.idl";
    let idl_src = std::fs::read_to_string(idl_path).expect("(;_;) Couldn't read IDL source file!");

    let result = parser::IdlParser::parse(parser::Rule::specification, &idl_src)
        .expect("Couldn't parse IDL file");

    let mut file = File::create("big_data.rs").expect("Failed to create file");

    let spec = syntax::specification()
        .analyse(result.into())
        .expect("Couldn't analyse IDL syntax");

    for def in spec.value {
        for line in rust::definition(def) {
            file.write_all(line.as_bytes())
                .expect("Failed to write to file");
        }
    }
}
