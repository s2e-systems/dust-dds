#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
struct Point {
    x: f64,
    y: f64,
}
#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
struct ChessSquare {
    column: char,
    line: u16,
}