use std::sync::{Arc, Mutex};

use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolPublisher, ProtocolSubscriber, ProtocolWriter, ProtocolReader};
use rust_dds_interface::qos::{DataWriterQos, DataReaderQos};

use crate::types::{GUID, Locator, GuidPrefix};
use crate::messages::RtpsSubmessage;
use crate::messages::message_sender::RtpsMessageSender;
use crate::structure::RtpsEndpoint;

pub struct RtpsGroup {
    guid: GUID,
    sender: RtpsMessageSender,
    endpoints: Vec<Arc<Mutex<dyn RtpsEndpoint>>>,
}

impl RtpsGroup {
    pub fn new(guid: GUID, sender: RtpsMessageSender) -> Self {
        Self {
            guid,
            sender,
            endpoints: Vec::new(),
        }
    }

    pub fn try_push_message(&self, _src_locator: Locator, _src_guid_prefix: GuidPrefix, _submessage: &mut Option<RtpsSubmessage>) {
        todo!()
    }
}

impl ProtocolEntity for RtpsGroup {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}

impl ProtocolPublisher for RtpsGroup {
    fn create_writer(&mut self, _topic_kind: TopicKind, _data_writer_qos: &DataWriterQos) -> Arc<Mutex<dyn ProtocolWriter>> {
        todo!()
    }
}

impl ProtocolSubscriber for RtpsGroup {
    fn create_reader(&mut self, _topic_kind: TopicKind, _data_reader_qos: &DataReaderQos) -> Arc<Mutex<dyn ProtocolReader>> {
        todo!()
    }
}
