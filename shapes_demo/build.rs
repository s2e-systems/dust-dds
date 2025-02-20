use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

fn main() {
    let cargo_target_dir = std::env::var("OUT_DIR").unwrap();
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_target_path = Path::new(&cargo_target_dir);
    let cargo_manifest_path = Path::new(&cargo_manifest_dir);
    let build_path = cargo_target_path.join("idl");
    let idl_path = cargo_manifest_path.join("res/ShapeType.idl");
    let compiled_idl = dust_dds_gen::compile_idl(&idl_path).expect("Couldn't parse IDL file");
    let compiled_idl_path = build_path.as_path().join("shapes_type.rs");
    fs::create_dir_all(build_path).expect("Creating build path failed");
    let mut file = File::create(compiled_idl_path).expect("Failed to create file");
    file.write_all(compiled_idl.as_bytes())
        .expect("Failed to write to file");
}
