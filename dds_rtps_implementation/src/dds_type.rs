pub trait DDSType: serde::Serialize + for<'de> serde::Deserialize<'de> {
    fn type_name() -> &'static str;

    fn has_key() -> bool;
}
