#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsHasKey, dust_dds::topic_definition::type_support::DdsGetKey, dust_dds::topic_definition::type_support::DdsRepresentation)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsHasKey, dust_dds::topic_definition::type_support::DdsGetKey, dust_dds::topic_definition::type_support::DdsRepresentation)]
pub struct ChessSquare {
    #[key] pub column: char,
    #[key] pub line: u16,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsHasKey, dust_dds::topic_definition::type_support::DdsGetKey, dust_dds::topic_definition::type_support::DdsRepresentation)]
pub struct HelloWorld {
    pub message: String,
    pub id: u32,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsHasKey, dust_dds::topic_definition::type_support::DdsGetKey, dust_dds::topic_definition::type_support::DdsRepresentation)]
pub struct Sentence {
    pub words: Vec<String>,
    pub dependencies: Vec<[u32; 2]>,
}
#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsHasKey, dust_dds::topic_definition::type_support::DdsGetKey, dust_dds::topic_definition::type_support::DdsRepresentation)]
pub struct User {
    pub name: String,
}