pub trait DDSType<PSM: crate::rtps_impl::PIM> {
    fn type_name() -> &'static str;

    fn has_key() -> bool;

    fn key(&self) -> PSM::InstanceHandle;

    fn serialize(&self) -> PSM::Data;

    fn deserialize(data: Vec<u8>) -> Self;
}
