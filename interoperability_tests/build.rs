use std::{fs::File, io::Write};

fn main() {
    let idl_path = "HelloWorld.idl";
    let idl_src = std::fs::read_to_string(idl_path).expect("(;_;) Couldn't read IDL source file!");

    let compiled_idl = dust_idlgen::compile_idl(&idl_src).expect("Couldn't parse IDL file");

    let mut file = File::create("hello_world.rs").expect("Failed to create file");

    file.write_all(compiled_idl.as_bytes())
        .expect("Failed to write to file");
}
