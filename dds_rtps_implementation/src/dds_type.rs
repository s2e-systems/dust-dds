pub trait DDSType {
    fn type_name() -> &'static str;

    fn has_key() -> bool;
}
