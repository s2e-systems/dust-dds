use std::sync::{Arc, Mutex};

// use crate::reader::Re
use crate::structure::RtpsGroup;
use crate::types::GUID;

use rust_dds_interface::protocol::{ProtocolSubscriber, ProtocolEntity, ProtocolReader};
use rust_dds_interface::types::{TopicKind, ReturnCode, InstanceHandle};
pub struct Subscriber {
    group: Arc<Mutex<RtpsGroup>>,
}

impl Subscriber {
    pub fn new(group: Arc<Mutex<RtpsGroup>>) -> Self {        
        Self {
            group
        }
    }
}

impl ProtocolEntity for Subscriber {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}
impl ProtocolSubscriber for Subscriber {
    fn create_reader(&mut self, topic_kind: TopicKind, data_reader_qos: &rust_dds_interface::qos::DataReaderQos) -> Box<dyn ProtocolReader> {
        todo!()
    }
}