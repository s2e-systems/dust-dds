use std::path::Path;

use syn::File;

#[test]
fn const_declarations() {
    let idl_file = Path::new("tests/const_declarations.idl");
    let expected_output = r#"
        pub const FOO: &str = "BAR";
    "#;
    let expected = syn::parse2::<File>(expected_output.parse().unwrap()).unwrap();

    let compiled_idl = dust_dds_gen::compile_idl(idl_file).unwrap();
    let result = syn::parse2::<File>(compiled_idl.parse().unwrap()).unwrap();

    assert_eq!(
        result, expected,
        "Expected: \n\n {expected_output}\n\n ======= Generated \n\n{compiled_idl}\n\n"
    );
}
