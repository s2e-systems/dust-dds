use std::path::Path;

use syn::File;

#[test]
fn appendable_struct() {
    let idl_file = Path::new("tests/appendable_struct.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "appendable")]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }

            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "mutable")]
            pub struct Data {
                #[dust_dds(key)]
                pub id: i16,
                pub x: f64,
            }

            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "appendable")]
            pub struct MultiDimensionalPoint {
                pub x: f64,
                pub y: f64,
                #[dust_dds(optional)]
                pub z: Option<f64>,
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
