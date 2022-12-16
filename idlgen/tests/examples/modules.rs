mod Game {
    mod Chess {
        #[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
        pub enum ChessPiece {
            Pawn,
            Rook,
            Knight,
            Bishop,
            Queen,
            King,
        }
        #[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
        pub struct ChessSquare {
            column: char,
            line: u16,
        }
    }
    mod Cards {
        #[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
        pub enum Suit {
            Spades,
            Hearts,
            Diamonds,
            Clubs,
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
pub struct Point {
    x: f64,
    y: f64,
}