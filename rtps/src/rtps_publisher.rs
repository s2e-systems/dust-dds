use crate::participant::Participant;
use crate::publisher::Publisher;

use rust_dds_interface::protocol::{ProtocolEntity, ProtocolPublisher, ProtocolWriter};
use rust_dds_interface::qos::DataWriterQos;
use rust_dds_interface::types::{TopicKind, ReturnCode};
pub struct RtpsPublisher<'a> {
    parent_participant: &'a Participant,
    publisher: &'a Publisher,
}

impl<'a> RtpsPublisher<'a> {
    pub fn new(parent_participant: &'a Participant, publisher: &'a Publisher) -> Self {
        Self {
            parent_participant,
            publisher,
        }
    }
}

impl<'a> ProtocolEntity for RtpsPublisher<'a> {
    fn get_instance_handle(&self) -> rust_dds_interface::types::InstanceHandle {
        todo!()
    }

    fn enable(&self) {
        todo!()
    }
}

impl<'a> ProtocolPublisher for RtpsPublisher<'a> {
    fn create_writer<'b>(&'b self, _topic_kind: TopicKind, _data_writer_qos: &DataWriterQos) -> ReturnCode<Box<dyn ProtocolWriter + 'b>> {
        todo!()
    }
}