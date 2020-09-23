use rust_dds_interface::types::{InstanceHandle, Data};

pub trait DDSType {
    fn instance_handle(&self) -> InstanceHandle;

    fn serialize(&self) -> Data;

    fn deserialize(data: Data) -> Self;
}

 
