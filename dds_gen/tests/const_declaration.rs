use std::path::Path;

use syn::File;

#[test]
fn const_declarations() {
    let idl_file = Path::new("tests/const_declarations.idl");
    let expected_output = r#"
        pub const MY_STRING: &str = "BAR";
        pub const MY_SHORT: i16 = -1;
        pub const MY_USHORT: u16 = 2;
        pub const MY_LONG: i32 = -3;
        pub const MY_ULONG: u32 = 4;
        pub const MY_LONGLONG: i64 = -5;
        pub const MY_ULONGLONG: u64 = 6;
        pub const MY_FLOAT: f32 = 1.5;
        pub const MY_DOUBLE: f64 = 2.5;
        pub const MY_CHAR: char = 'a';
        pub const MY_BOOL: bool = TRUE;
        pub const MY_OCTET: u8 = 7;
        pub const MY_INT8: i8 = -8;
        pub const MY_UINT8: u8 = 8;
        pub const MY_INT16: i16 = -16;
        pub const MY_UINT16: u16 = 16;
        pub const MY_INT32: i32 = -32;
        pub const MY_UINT32: u32 = 32;
        pub const MY_INT64: i64 = -64;
        pub const MY_UINT64: u64 = 64;
    "#;
    let expected = syn::parse2::<File>(expected_output.parse().unwrap()).unwrap();

    let compiled_idl = dust_dds_gen::compile_idl(idl_file).unwrap();
    let result = syn::parse2::<File>(compiled_idl.parse().unwrap()).unwrap();

    assert_eq!(
        result, expected,
        "Expected: \n\n {expected_output}\n\n ======= Generated \n\n{compiled_idl}\n\n"
    );
}
