use syn::File;

#[test]
fn dds_idl_compilation() {
    let idl = "
        #define BASIC_TYPE long

        struct TemplateTypes {
            BASIC_TYPE a
        };
    ";

    let expected = syn::parse2::<File>(r#""#.parse().unwrap()).unwrap();

    let result =
        syn::parse2::<File>(dust_dds_gen::compile_idl(idl).unwrap().parse().unwrap()).unwrap();

    assert_eq!(result, expected);
}
