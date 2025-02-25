# Dust DDS IDLgen

A tool that generates Rust code for Dust DDS from IDL files. If using only Rust, you can make use of the procedural macros to enable a type to be transmitted using Dust DDS. This crate is useful in all cases where you want to use different programming languages or vendors or want to have the types defined externally to the code in idl format.

Some of the relevant features of Dust DDS IDLgen:

- Supports `@key` definition on the struct fields
- Allows using preprocessor directives `#define`, `#include`, `#ifdef` and `#ifndef`

## Usage

This IDL gen is shipped as a library instead of a self standing tool since its main use case is to be integrated with a `build.rs` script. The entry function is `dust_dds_gen::compile_idl(&idl_path)` which generates a string with the output Rust code. Here is an example of a 'build.rs' file using the IDL generator:

```rust
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
    file.write_all(compiled_idl.as_bytes()).expect("Failed to write to file");
}
```

Once the file is generated it can be included in the code by using the `include!` macro:

```rust
include!(concat!(env!("OUT_DIR"), "/idl/shapes_type.rs"));
```

## License

This project is licensed under the Apache License Version 2.0.
