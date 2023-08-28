#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsType, dust_dds::topic_definition::type_support::DdsKey)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsType, dust_dds::topic_definition::type_support::DdsKey)]
pub struct ChessSquare {
    #[key] pub column: char,
    #[key] pub line: u16,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsType, dust_dds::topic_definition::type_support::DdsKey)]
pub struct HelloWorld {
    pub message: String,
    pub id: u32,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsType, dust_dds::topic_definition::type_support::DdsKey)]
pub struct Sentence {
    pub words: Vec<String>,
    pub dependencies: Vec<[u32; 2]>,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsType, dust_dds::topic_definition::type_support::DdsKey)]
pub struct User {
    pub name: String,
}