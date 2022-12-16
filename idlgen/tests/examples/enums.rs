#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
pub enum Suits {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}
#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
pub enum Direction {
    North,
    East,
    South,
    West,
}