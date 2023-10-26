mod Game {
    mod Chess {
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
    mod Cards {
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