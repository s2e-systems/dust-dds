use rust_dds_api::{
    dcps_psm::{InstanceHandle, Time},
    dds_type::DDSType,
    return_type::DDSResult,
};

pub struct DataWriterImpl;

impl DataWriterImpl {
    pub fn register_instance_w_timestamp<T: DDSType>(
        &mut self,
        _instance: T,
        _timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }
}
