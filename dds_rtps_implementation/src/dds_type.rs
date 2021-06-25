use rust_rtps_pim::structure::types::InstanceHandlePIM;

pub trait DDSType<PSM: InstanceHandlePIM> {
    fn type_name() -> &'static str;

    fn has_key() -> bool;

    fn key(&self) -> PSM::InstanceHandleType;

    fn serialize(&self) -> Vec<u8>;

    fn deserialize(data: Vec<u8>) -> Self;
}
