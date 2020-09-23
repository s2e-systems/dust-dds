pub use rust_dds_interface::types::{InstanceHandle, Data, Duration, DURATION_INFINITE, DURATION_ZERO, Time, TIME_INVALID, LENGTH_UNLIMITED, ReturnCode, ReturnCodes};






pub trait DDSType {
    fn instance_handle(&self) -> InstanceHandle;

    fn serialize(&self) -> Data;

    fn deserialize(data: Data) -> Self;
}

 
