pub trait DDSType: serde::Serialize {
    fn type_name() -> &'static str;

    fn has_key() -> bool;
}
