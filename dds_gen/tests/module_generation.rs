use std::path::Path;

use syn::File;

#[test]
fn module_generation() {
    let idl_file = Path::new("tests/module_generation.idl");

    let expected_string = r#"
        pub mod Game {
            pub mod Chess {
                #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
                #[dust_dds(name = "Game::Chess::ChessPiece")]
                pub enum ChessPiece {
                    Pawn,
                    Rook,
                    Knight,
                    Bishop,
                    Queen,
                    King,
                }
                #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
                #[dust_dds(name = "Game::Chess::ChessSquare")]
                pub struct ChessSquare {
                    pub column: char,
                    pub line: u16,
                }
            }
            pub mod Cards {
                #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
                #[dust_dds(name = "Game::Cards::Suit")]
                pub enum Suit {
                    Spades,
                    Hearts,
                    Diamonds,
                    Clubs,
                }
            }
        }
        #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
        pub struct Point {
            pub x: f64,
            pub y: f64,
        }

        pub mod foo{
            pub type Bar=i32;
            pub type Car=i32;
            pub mod frob{
                #[derive(Debug, Clone, dust_dds::infrastructure::type_support::DdsType)]
                #[dust_dds(name = "foo::frob::Baz")]
                pub struct Baz {
                    #[dust_dds(key)]
                    pub qux: super::super::foo::Bar,
                    pub qix: super::super::foo::Car,
                }
            }
        }
    "#;
    let expected = syn::parse2::<File>(expected_string.parse().unwrap()).unwrap();

    let compiled_idl = dust_dds_gen::compile_idl(idl_file).unwrap();
    let result = syn::parse2::<File>(compiled_idl.parse().unwrap()).unwrap();

    assert_eq!(
        result, expected,
        "Expected: \n\n {expected_string} \n\n ====== Generated: \n\n {compiled_idl}"
    );
}
