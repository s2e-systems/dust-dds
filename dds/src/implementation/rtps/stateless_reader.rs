use crate::{
    implementation::dds_impl::message_receiver::MessageReceiver,
    infrastructure::{
        instance::InstanceHandle, qos_policy::ReliabilityQosPolicyKind,
        status::SampleRejectedStatusKind,
    },
};

use super::{
    messages::submessages::DataSubmessage,
    reader::{RtpsReader, RtpsReaderError},
    types::ENTITYID_UNKNOWN,
};

pub struct RtpsStatelessReader(RtpsReader);

pub enum StatelessReaderDataReceivedResult {
    NotForThisReader,
    NewSampleAdded(InstanceHandle),
    SampleRejected(InstanceHandle, SampleRejectedStatusKind),
    InvalidData(&'static str),
}

impl RtpsStatelessReader {
    pub fn new(reader: RtpsReader) -> Self {
        if reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            panic!("Reliable stateless reader is not supported");
        }

        Self(reader)
    }

    pub fn reader(&self) -> &RtpsReader {
        &self.0
    }

    pub fn reader_mut(&mut self) -> &mut RtpsReader {
        &mut self.0
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> StatelessReaderDataReceivedResult {
        if data_submessage.reader_id == ENTITYID_UNKNOWN
            || data_submessage.reader_id == self.0.guid().entity_id()
        {
            let add_change_result = self.0.add_change(
                data_submessage,
                Some(message_receiver.timestamp()),
                message_receiver.source_guid_prefix(),
                message_receiver.reception_timestamp(),
            );

            match add_change_result {
                Ok(h) => StatelessReaderDataReceivedResult::NewSampleAdded(h),
                Err(e) => match e {
                    RtpsReaderError::InvalidData(s) => {
                        StatelessReaderDataReceivedResult::InvalidData(s)
                    }
                    RtpsReaderError::Rejected(h, k) => {
                        StatelessReaderDataReceivedResult::SampleRejected(h, k)
                    }
                },
            }
        } else {
            StatelessReaderDataReceivedResult::NotForThisReader
        }
    }
}
