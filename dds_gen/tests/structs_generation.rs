use std::path::Path;

use syn::File;

#[test]
fn structs_generation() {
    let idl_file = Path::new("tests/structs_generation.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct ChessSquare {
                #[dust_dds(key)] pub column: char,
                #[dust_dds(key)] pub line: u16,
            }
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct HelloWorld {
                pub message: String,
                pub id: u32,
            }
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Sentence {
                pub words: Vec<String>,
                pub dependencies: Vec<Vec<u32>>,
            }
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct User {
                pub name: String,
                pub active: bool,
            }
            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct ObjectiveType {
                pub name: String,
            }

            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Objective {
                pub objective_type: ObjectiveType,
            }

            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Parent {
                pub a: u8,
            }

            #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(base_type = Parent)]
            pub struct Child {
                pub parent: Parent,
                pub name: String,
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
