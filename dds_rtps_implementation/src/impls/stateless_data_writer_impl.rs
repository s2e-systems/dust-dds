use rust_dds_api::{
    dcps_psm::{InstanceHandle, Time},
    dds_type::DDSType,
    return_type::DDSResult,
};

pub struct StatelessDataWriterImpl {}

impl StatelessDataWriterImpl {
    pub fn new() -> Self {
        Self {}
    }

    pub fn register_instance_w_timestamp<T: DDSType>(
        &mut self,
        _instance: T,
        _timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    pub fn write_w_timestamp<T: DDSType>(
        &mut self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        // let kind = ChangeKind::Alive;
        // let data = data.serialize();
        // let inline_qos = MyParameterList::new();
        // let handle = handle.unwrap_or(0);
        // let change = self.writer.new_change(kind, data, inline_qos, handle);

        // self.writer.writer_cache_mut().add_change(change);

        Ok(())
    }
}
