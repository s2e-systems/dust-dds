use std::path::Path;

use syn::File;

#[test]
fn appendable_struct() {
    let idl_file = Path::new("tests/appendable_struct.idl");

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "appendable")]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }

            #[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
            #[dust_dds(extensibility = "mutable")]
            pub struct Data {
                #[dust_dds(key)]
                pub id: i16,
                pub x: f64,
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
