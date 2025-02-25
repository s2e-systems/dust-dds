# Dust DDS IDLgen

A tool that generates Rust code for Dust DDS from IDL files. If using only Rust, you can make use of the procedural macros to enable a type to be transmitted using Dust DDS. This crate is useful in all cases where you want to use different programming languages or vendors or want to have the types defined externally to the code in idl format.

Some of the relevant features of Dust DDS IDLgen:

- Supports `@key` definition on the struct fields
- Allows using preprocessor directives `#define`, `#include`, `#ifdef` and `#ifndef`

## Usage

This IDL gen is shipped as a library instead of a self standing tool since its main use case is to be integrated with a `build.rs` script. The entry function is `dust_dds_gen::compile_idl(&idl_path)` which generates a string with the output Rust code. Here is an example of an idl file:

```idl
struct HelloWorldType {
  @key
  uint8 id;
  string msg;
};
```

## License

This project is licensed under the Apache License Version 2.0.
