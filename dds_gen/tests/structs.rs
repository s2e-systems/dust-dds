use syn::File;

#[test]
fn structs_generation() {
    let idl = r#"
        struct Point {
            double x;
            double y;
        };

        struct ChessSquare {
            @key char column;
            @key unsigned short line;
        };

        struct HelloWorld {
            string message;
            unsigned long id;
        };

        struct Sentence {
            sequence<wstring> words;
            sequence<sequence<unsigned long, 2> > dependencies;
        };

        struct User {
            wstring<8> name;
            boolean active;
        };
    "#;

    let expected = syn::parse2::<File>(
        r#"
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
            pub struct ChessSquare {
                #[dust_dds(key)] pub column: char,
                #[dust_dds(key)] pub line: u16,
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
            pub struct HelloWorld {
                pub message: String,
                pub id: u32,
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
            pub struct Sentence {
                pub words: Vec<String>,
                pub dependencies: Vec<Vec<u32>>,
            }
            #[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
            pub struct User {
                pub name: String,
                pub active: bool,
            }
    "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let result =
        syn::parse2::<File>(dust_dds_gen::compile_idl(idl).unwrap().parse().unwrap()).unwrap();

    assert_eq!(result, expected);
}

#[test]
fn module_generation() {
    let idl = r#"
        /*
        * Module IDL example
        */

        /// Game module
        module Game
        {
            /// Chess
            module Chess
            {
                enum ChessPiece
                {
                    Pawn, Rook, Knight, Bishop, Queen, King
                };

                struct ChessSquare
                {
                    char column;          // A, B, ..., G
                    unsigned short line;  // 1, 2, ..., 8
                };
            };

            module Cards
            {
                enum Suit { Spades, Hearts, Diamonds, Clubs /*, NoTrumpCard */ };
            };
        };

        struct Point
        {
            double x;
            double y;
        };
    "#;

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
                #[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
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
        #[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
        pub struct Point {
            pub x: f64,
            pub y: f64,
        }
    "#
        .parse()
        .unwrap(),
    )
    .unwrap();

    let result =
        syn::parse2::<File>(dust_dds_gen::compile_idl(idl).unwrap().parse().unwrap()).unwrap();

    assert_eq!(result, expected);
}
