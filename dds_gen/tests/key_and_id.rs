use std::path::Path;

use syn::File;

#[test]
fn key_and_id_attributes() {
    let idl_file = Path::new("tests/key_and_id.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Message {
                #[dust_dds(key)]
                #[dust_dds(id = 1)]
                pub id: i32,
                #[dust_dds(id = 2)]
                pub content: String,
                #[dust_dds(id = 3)]
                pub timestamp: i64,
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
