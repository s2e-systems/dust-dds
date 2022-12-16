#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
pub struct ChessSquare {
    pub column: char,
    pub line: u16,
}