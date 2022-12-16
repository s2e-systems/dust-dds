#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
enum Suits {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}
#[derive(serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
enum Direction {
    North,
    East,
    South,
    West,
}