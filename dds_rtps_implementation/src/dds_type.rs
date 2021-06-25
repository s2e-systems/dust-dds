pub trait DDSType {
    fn type_name() -> &'static str;

    fn has_key() -> bool;

    fn key(&self) -> i32;

    fn serialize(&self) -> Vec<u8>;

    fn deserialize(data: Vec<u8>) -> Self;
}
