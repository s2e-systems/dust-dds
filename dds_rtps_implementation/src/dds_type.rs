use rust_rtps_pim::structure::types::{DataPIM, InstanceHandlePIM};

pub trait DDSType<PSM: InstanceHandlePIM + DataPIM> {
    fn type_name() -> &'static str;

    fn has_key() -> bool;

    fn key(&self) -> PSM::InstanceHandleType;

    fn serialize(&self) -> PSM::DataType;

    fn deserialize(data: Vec<u8>) -> Self;
}
