mod Game {
    mod Chess {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub enum ChessPiece {
            Pawn,
            Rook,
            Knight,
            Bishop,
            Queen,
            King,
        }
        #[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsHasKey, dust_dds::topic_definition::type_support::DdsGetKey, dust_dds::topic_definition::type_support::DdsRepresentation)]
        pub struct ChessSquare {
            pub column: char,
            pub line: u16,
        }
    }
    mod Cards {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub enum Suit {
            Spades,
            Hearts,
            Diamonds,
            Clubs,
        }
    }
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsHasKey, dust_dds::topic_definition::type_support::DdsGetKey, dust_dds::topic_definition::type_support::DdsRepresentation)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}