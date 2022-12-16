mod Game {
    mod Chess {
        enum ChessPiece {
            Pawn,
            Rook,
            Knight,
            Bishop,
            Queen,
            King,
        }
        struct ChessSquare {
            column: char,
            line: u16,
        }
    }
    mod Cards {
        enum Suit {
            Spades,
            Hearts,
            Diamonds,
            Clubs,
        }
    }
}
struct Point {
    x: f64,
    y: f64,
}