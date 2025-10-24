use std::path::Path;

use syn::File;

#[test]
fn template_types() {
    let idl_file = Path::new("tests/template_types.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct TemplateTypes {
                pub a: Vec<Vec<u8>>,
                pub b: String,
                pub c: Vec<i16>,
                pub d: String,
            }
    "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let result = syn::parse2::<File>(
        dust_dds_gen::compile_idl(idl_file)
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(result, expected);
}
