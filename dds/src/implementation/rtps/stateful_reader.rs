use crate::{
    implementation::dds_impl::message_receiver::MessageReceiver,
    infrastructure::{
        instance::InstanceHandle,
        qos_policy::ReliabilityQosPolicyKind,
        status::SampleRejectedStatusKind,
        time::{Duration, DURATION_ZERO},
    },
};

use super::{
    messages::{
        overall_structure::RtpsMessageHeader,
        submessage_elements::{
            EntityIdSubmessageElement, GuidPrefixSubmessageElement,
            ProtocolVersionSubmessageElement, SequenceNumberSetSubmessageElement,
            VendorIdSubmessageElement,
        },
        submessages::{
            AckNackSubmessage, DataSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
        },
        types::ProtocolId,
        RtpsMessage, RtpsSubmessageKind,
    },
    reader::{RtpsReader, RtpsReaderError},
    transport::TransportWrite,
    types::{Guid, GuidPrefix, PROTOCOLVERSION, VENDOR_ID_S2E},
    writer_proxy::RtpsWriterProxy,
};

pub const DEFAULT_HEARTBEAT_RESPONSE_DELAY: Duration = Duration::new(0, 500);
pub const DEFAULT_HEARTBEAT_SUPPRESSION_DURATION: Duration = DURATION_ZERO;

/// ChangeFromWriterStatusKind
/// Enumeration used to indicate the status of a ChangeFromWriter. It can take the values:
/// LOST, MISSING, RECEIVED, UNKNOWN
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeFromWriterStatusKind {
    Lost,
    Missing,
    Received,
    Unknown,
}

pub enum StatefulReaderDataReceivedResult {
    NoMatchedWriterProxy,
    UnexpectedDataSequenceNumber,
    NewSampleAdded(InstanceHandle),
    NewSampleAddedAndSamplesLost(InstanceHandle),
    SampleRejected(InstanceHandle, SampleRejectedStatusKind),
    InvalidData(&'static str),
}

impl From<RtpsReaderError> for StatefulReaderDataReceivedResult {
    fn from(e: RtpsReaderError) -> Self {
        match e {
            RtpsReaderError::InvalidData(s) => StatefulReaderDataReceivedResult::InvalidData(s),
            RtpsReaderError::Rejected(instance_handle, reason) => {
                StatefulReaderDataReceivedResult::SampleRejected(instance_handle, reason)
            }
        }
    }
}

pub struct RtpsStatefulReader {
    reader: RtpsReader,
    matched_writers: Vec<RtpsWriterProxy>,
}

impl RtpsStatefulReader {
    pub fn new(reader: RtpsReader) -> Self {
        Self {
            reader,
            matched_writers: Vec::new(),
        }
    }

    pub fn reader(&self) -> &RtpsReader {
        &self.reader
    }

    pub fn reader_mut(&mut self) -> &mut RtpsReader {
        &mut self.reader
    }

    pub fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy) {
        if !self
            .matched_writers
            .iter()
            .any(|x| x.remote_writer_guid() == a_writer_proxy.remote_writer_guid())
        {
            self.matched_writers.push(a_writer_proxy);
        }
    }

    pub fn matched_writer_remove(&mut self, a_writer_guid: Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != a_writer_guid)
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> StatefulReaderDataReceivedResult {
        let sequence_number = data_submessage.writer_sn.value;
        let writer_guid = Guid::new(
            message_receiver.source_guid_prefix(),
            data_submessage.writer_id.value,
        );

        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_guid)
        {
            let expected_seq_num = writer_proxy.available_changes_max() + 1;
            match self.reader.get_qos().reliability.kind {
                ReliabilityQosPolicyKind::BestEffort => {
                    if sequence_number >= expected_seq_num {
                        let add_change_result = self.reader.add_change(
                            data_submessage,
                            Some(message_receiver.timestamp()),
                            message_receiver.source_guid_prefix(),
                            message_receiver.reception_timestamp(),
                        );

                        match add_change_result {
                            Ok(instance_handle) => {
                                writer_proxy.received_change_set(sequence_number);
                                if sequence_number > expected_seq_num {
                                    writer_proxy.lost_changes_update(sequence_number);
                                    StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(
                                        instance_handle,
                                    )
                                } else {
                                    StatefulReaderDataReceivedResult::NewSampleAdded(
                                        instance_handle,
                                    )
                                }
                            }
                            Err(err) => err.into(),
                        }
                    } else {
                        StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    if sequence_number == expected_seq_num {
                        let add_change_result = self.reader.add_change(
                            data_submessage,
                            Some(message_receiver.timestamp()),
                            message_receiver.source_guid_prefix(),
                            message_receiver.reception_timestamp(),
                        );

                        match add_change_result {
                            Ok(instance_handle) => {
                                writer_proxy.received_change_set(data_submessage.writer_sn.value);
                                StatefulReaderDataReceivedResult::NewSampleAdded(instance_handle)
                            }
                            Err(err) => err.into(),
                        }
                    } else {
                        StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
            }
        } else {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy
        }
    }

    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_submessage.writer_id.value);

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count() < heartbeat_submessage.count {
                    writer_proxy.set_last_received_heartbeat_count(heartbeat_submessage.count);

                    writer_proxy.set_must_send_acknacks(
                        !heartbeat_submessage.final_flag
                            || (!heartbeat_submessage.liveliness_flag
                                && !writer_proxy.missing_changes().is_empty()),
                    );

                    if !heartbeat_submessage.final_flag {
                        writer_proxy.set_must_send_acknacks(true);
                    }
                    writer_proxy.missing_changes_update(heartbeat_submessage.last_sn.value);
                    writer_proxy.lost_changes_update(heartbeat_submessage.first_sn.value);
                }
            }
        }
    }

    pub fn send_message(&mut self, transport: &mut impl TransportWrite) {
        for writer_proxy in self.matched_writers.iter_mut() {
            if writer_proxy.must_send_acknacks() || !writer_proxy.missing_changes().is_empty() {
                writer_proxy.set_must_send_acknacks(false);
                writer_proxy.increment_acknack_count();

                let info_dst_submessage = InfoDestinationSubmessage {
                    endianness_flag: true,
                    guid_prefix: GuidPrefixSubmessageElement {
                        value: writer_proxy.remote_writer_guid().prefix(),
                    },
                };

                let acknack_submessage = AckNackSubmessage {
                    endianness_flag: true,
                    final_flag: true,
                    reader_id: EntityIdSubmessageElement {
                        value: self.reader.guid().entity_id(),
                    },
                    writer_id: EntityIdSubmessageElement {
                        value: writer_proxy.remote_writer_guid().entity_id(),
                    },
                    reader_sn_state: SequenceNumberSetSubmessageElement {
                        base: writer_proxy.available_changes_max() + 1,
                        set: writer_proxy.missing_changes(),
                    },
                    count: writer_proxy.acknack_count(),
                };

                let header = RtpsMessageHeader {
                    protocol: ProtocolId::PROTOCOL_RTPS,
                    version: ProtocolVersionSubmessageElement {
                        value: PROTOCOLVERSION,
                    },
                    vendor_id: VendorIdSubmessageElement {
                        value: VENDOR_ID_S2E,
                    },
                    guid_prefix: GuidPrefixSubmessageElement {
                        value: self.reader.guid().prefix(),
                    },
                };

                let message = RtpsMessage {
                    header,
                    submessages: vec![
                        RtpsSubmessageKind::InfoDestination(info_dst_submessage),
                        RtpsSubmessageKind::AckNack(acknack_submessage),
                    ],
                };

                for locator in writer_proxy.unicast_locator_list() {
                    transport.write(&message, *locator);
                }
            }
        }
    }
}
