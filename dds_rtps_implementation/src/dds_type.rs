pub trait DdsType {
    fn type_name() -> &'static str;

    fn has_key() -> bool;
}

pub trait DdsSerialize {

}
