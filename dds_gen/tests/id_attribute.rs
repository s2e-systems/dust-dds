use std::path::Path;

use syn::File;

#[test]
fn id_attribute() {
    let idl_file = Path::new("tests/id_attribute.idl");

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
