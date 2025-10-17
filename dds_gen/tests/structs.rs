use std::path::Path;

use syn::File;
use syn::__private::ToTokens;

#[test]
fn structs_generation() {
    let idl_file = Path::new("tests/structs_generation.idl");

    let expected = syn::parse2::<File>(
            r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct ChessSquare {
                #[dust_dds(key)] pub column: char,
                #[dust_dds(key)] pub line: u16,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct HelloWorld {
                pub message: String,
                pub id: u32,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Sentence {
                pub words: Vec<String>,
                pub dependencies: Vec<Vec<u32>>,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct User {
                pub name: String,
                pub active: bool,
            }
        "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let compiled = dust_dds_gen::compile_idl(idl_file);
    println!("{:?}",compiled);

    let result = syn::parse2::<File>(
        compiled
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(result, expected);
}

#[test]
fn module_generation() {
    let idl_file = Path::new("tests/module_generation.idl");

    let expected = syn::parse2::<File>(
        r#"
        pub mod Game {
            pub mod Chess {
                #[derive(Debug)]
                pub enum ChessPiece {
                    Pawn,
                    Rook,
                    Knight,
                    Bishop,
                    Queen,
                    King,
                }
                #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
                pub struct ChessSquare {
                    pub column: char,
                    pub line: u16,
                }
            }
            pub mod Cards {
                #[derive(Debug)]
                pub enum Suit {
                    Spades,
                    Hearts,
                    Diamonds,
                    Clubs,
                }
            }
        }
        #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
        pub struct Point {
            pub x: f64,
            pub y: f64,
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

#[test]
fn nested_types() {
    let idl_file = Path::new("tests/nested_types.idl");

    let expected = syn::parse2::<File>(
            r#"
            #[derive(Debug)]
            pub enum Presence {
                Present,
                NotPresent,
            }
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct Color {
                pub red: u8,
                pub green: u8,
                pub blue: u8,
            }

            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            pub struct ColorSensor {
                pub state: Presence,
                pub value: Color,
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

#[test]
fn test_generate_rust_code_with_annotations() {
    let idl_file = Path::new("tests/annotations.idl");

    let expected = syn::parse2::<File>(
            r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "appendable")]
            pub struct MyStruct {
                pub data: Vec<u8>,
                #[serde(skip)]
                pub ignored_field: i32,
            }
        "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let compiled = dust_dds_gen::compile_idl(idl_file);
    println!("{:?}",compiled);

    let result = syn::parse2::<File>(
        compiled
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    assert_eq!(result, expected);
}

#[test]
fn test_generate_rust_code_with_psm_annotations() {
    let idl_file = Path::new("tests/psm_annotations.idl");

    let expected_annotations = [
        "#[dust_dds(key)]",
        "#[dust_dds(extensibility = \"appendable\")]",
        "#[dust_dds(extensibility = \"mutable\")]",
        "#[dust_dds(extensibility = \"final\")]",
        "#[dust_dds(hashid)]",
        "#[dust_dds(must_understand)]",
        "#[dust_dds(optional)]",
    ];

    let compiled = dust_dds_gen::compile_idl(idl_file);
    println!("{:?}",compiled);

    let result = syn::parse2::<File>(
        compiled
            .unwrap()
            .parse()
            .unwrap(),
    )
    .unwrap();

    println!("{}", result.to_token_stream());


    fn file_contains(file: &File, keywords: &[&str]) -> bool {
        let code_string = file.to_token_stream().to_string();
        keywords.iter().all(|kw| code_string.contains(kw))
    }

    for ann in &expected_annotations {
        let normalized_keywords: Vec<&str> = ann
            .trim_matches(|c| c == '#' || c == '[' || c == ']')
            .split(|c| c == '(' || c == ')' || c == '=' || c == '"' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();

        assert!(
            file_contains(&result, &normalized_keywords),
            "Expected annotation {} missing",
            ann
        );
    }
}
