#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Suits {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
