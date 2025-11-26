use std::path::Path;

use syn::File;

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
fn module_generation() {
    let idl_file = Path::new("tests/module_generation.idl");

    let expected = syn::parse2::<File>(
        r#"
        pub mod Game {
            pub mod Chess {
                #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
                #[dust_dds(name = "Game::Chess::ChessPiece")]
                pub enum ChessPiece {
                    Pawn,
                    Rook,
                    Knight,
                    Bishop,
                    Queen,
                    King,
                }
                #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
                #[dust_dds(name = "Game::Chess::ChessSquare")]
                pub struct ChessSquare {
                    pub column: char,
                    pub line: u16,
                }
            }
            pub mod Cards {
                #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
                #[dust_dds(name = "Game::Cards::Suit")]
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
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
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
            }"#
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
