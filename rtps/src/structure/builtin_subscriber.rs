use rust_dds_interface::protocol::{ProtocolEntity, ProtocolSubscriber, ProtocolReader};

use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::qos::DataReaderQos;

pub struct BuiltinSubscriber;


impl ProtocolEntity for BuiltinSubscriber {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}

impl ProtocolSubscriber for BuiltinSubscriber {
    fn create_reader(&self, _topic_kind: TopicKind, _data_reader_qos: &DataReaderQos) -> std::sync::Arc<dyn ProtocolReader> {
        todo!()
    }
}