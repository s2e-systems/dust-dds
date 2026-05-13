use std::path::Path;

use syn::File;

#[test]
fn id_attributes() {
    let idl_file = Path::new("tests/id_attributes.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Person {
                #[dust_dds(id = 1)]
                pub name: String,
                #[dust_dds(id = 2)]
                pub age: i32,
                #[dust_dds(id = 3)]
                pub height: f64,
            }

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
