use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

fn main() {
    let build_path = Path::new("./target/idl/");
    fs::create_dir_all(build_path).expect("Creating build path failed");

    let idl_path = Path::new("HelloWorld.idl");
    let compiled_idl = dust_dds_gen::compile_idl(&idl_path).expect("Couldn't parse IDL file");
    let compiled_idl_path = build_path.join("hello_world.rs");
    let mut file = File::create(compiled_idl_path).expect("Failed to create file");
    file.write_all(compiled_idl.as_bytes())
        .expect("Failed to write to file");

    let idl_path = Path::new("BigData.idl");
    let compiled_idl = dust_dds_gen::compile_idl(&idl_path).expect("Couldn't parse IDL file");
    let compiled_idl_path = build_path.join("big_data.rs");
    let mut file = File::create(compiled_idl_path).expect("Failed to create file");
    file.write_all(compiled_idl.as_bytes())
        .expect("Failed to write to file");

    let idl_path = Path::new("DisposeData.idl");
    let compiled_idl = dust_dds_gen::compile_idl(&idl_path).expect("Couldn't parse IDL file");
    let compiled_idl_path = build_path.join("dispose_data.rs");
    let mut file = File::create(compiled_idl_path).expect("Failed to create file");
    file.write_all(compiled_idl.as_bytes())
        .expect("Failed to write to file");

    let idl_path = Path::new("NestedType.idl");
    let compiled_idl = dust_dds_gen::compile_idl(&idl_path).expect("Couldn't parse IDL file");
    let compiled_idl_path = build_path.join("nested_type.rs");
    let mut file = File::create(compiled_idl_path).expect("Failed to create file");
    file.write_all(compiled_idl.as_bytes())
        .expect("Failed to write to file");

    let idl_path = Path::new("AbsoluteData.idl");
    let compiled_idl = dust_dds_gen::compile_idl(&idl_path).expect("Couldn't parse IDL file");
    let compiled_idl_path = build_path.join("absolute_data.rs");
    let mut file = File::create(compiled_idl_path).expect("Failed to create file");
    file.write_all(compiled_idl.as_bytes())
        .expect("Failed to write to file");
}
