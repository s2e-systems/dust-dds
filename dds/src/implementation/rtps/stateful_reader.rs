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
        submessage_elements::SequenceNumberSet,
        submessages::{
            AckNackSubmessage, DataFragSubmessage, DataSubmessage, HeartbeatSubmessage,
            InfoDestinationSubmessage,
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
    // frag_buffer: Vec<DataFragSubmessage>
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
        let sequence_number = data_submessage.writer_sn;
        let writer_guid = Guid::new(
            message_receiver.source_guid_prefix(),
            data_submessage.writer_id,
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
                        if let Ok(change) = self.reader.convert_data_to_cache_change(
                            data_submessage,
                            Some(message_receiver.timestamp()),
                            message_receiver.source_guid_prefix(),
                            message_receiver.reception_timestamp(),
                        ) {
                            let add_change_result = self.reader.add_change(change);

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
                            todo!()
                        }
                    } else {
                        StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    if sequence_number == expected_seq_num {
                        if let Ok(change) = self.reader.convert_data_to_cache_change(
                            data_submessage,
                            Some(message_receiver.timestamp()),
                            message_receiver.source_guid_prefix(),
                            message_receiver.reception_timestamp(),
                        ) {
                            let add_change_result = self.reader.add_change(change);

                            match add_change_result {
                                Ok(instance_handle) => {
                                    writer_proxy.received_change_set(data_submessage.writer_sn);
                                    StatefulReaderDataReceivedResult::NewSampleAdded(
                                        instance_handle,
                                    )
                                }
                                Err(err) => err.into(),
                            }
                        } else {
                            todo!()
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

    pub fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> StatefulReaderDataReceivedResult {
        let sequence_number = data_frag_submessage.writer_sn;
        let writer_guid = Guid::new(
            message_receiver.source_guid_prefix(),
            data_frag_submessage.writer_id,
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
                        todo!()
                    } else {
                        StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    if sequence_number == expected_seq_num {
                        todo!()
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
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_submessage.writer_id);

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
                    writer_proxy.missing_changes_update(heartbeat_submessage.last_sn);
                    writer_proxy.lost_changes_update(heartbeat_submessage.first_sn);
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
                    guid_prefix: writer_proxy.remote_writer_guid().prefix(),
                };

                let acknack_submessage = AckNackSubmessage {
                    endianness_flag: true,
                    final_flag: true,
                    reader_id: self.reader.guid().entity_id(),
                    writer_id: writer_proxy.remote_writer_guid().entity_id(),
                    reader_sn_state: SequenceNumberSet {
                        base: writer_proxy.available_changes_max() + 1,
                        set: writer_proxy.missing_changes(),
                    },
                    count: writer_proxy.acknack_count(),
                };

                let header = RtpsMessageHeader {
                    protocol: ProtocolId::PROTOCOL_RTPS,
                    version: PROTOCOLVERSION,
                    vendor_id: VENDOR_ID_S2E,
                    guid_prefix: self.reader.guid().prefix(),
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

// #[cfg(test)]
// mod tests {
//     use crate::{
//         implementation::rtps::{
//             endpoint::RtpsEndpoint,
//             messages::types::{FragmentNumber, UShort},
//             types::{
//                 EntityId, EntityKey, LocatorAddress, LocatorKind, LocatorPort, TopicKind,
//                 ENTITYID_UNKNOWN, GUID_UNKNOWN, USER_DEFINED_READER_NO_KEY,
//             },
//             utils::clock::StdTimer,
//         },
//         topic_definition::type_support::{DdsSerde, DdsType},
//     };

//     use super::*;
//     #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
//     struct LargeData {
//         value: Vec<u8>,
//     }
//     impl DdsType for LargeData {
//         fn type_name() -> &'static str {
//             "LargeData"
//         }
//     }
//     impl DdsSerde for LargeData {}

//     use mockall::mock;

//     mock! {
//         Transport{}

//         impl TransportWrite for Transport {
//             fn write<'a>(&'a mut self, message: &RtpsMessage<'a>, destination_locator: Locator);
//         }
//     }

//     #[test]
//     fn read_frag_message() {
//         let remote_reader_guid = Guid::new(
//             GuidPrefix::new([4; 12]),
//             EntityId::new(EntityKey::new([0, 0, 0x03]), USER_DEFINED_READER_NO_KEY),
//         );

//         let max_bytes = 60000;

//         let mut rtps_writer = RtpsStatefulWriter::<StdTimer>::new(RtpsWriter::new(
//             RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
//             true,
//             DURATION_ZERO,
//             DURATION_ZERO,
//             DURATION_ZERO,
//             Some(max_bytes),
//             DataWriterQos::default(),
//         ));
//         let data = LargeData {
//             value: vec![3; 100000],
//         };
//         let data_length = data.value.len() + 4 /*CDR header*/ + 4 /*length of vector*/;

//         rtps_writer
//             .write_w_timestamp(&data, None, Time::new(1, 0))
//             .unwrap();

//         let remote_group_entity_id = ENTITYID_UNKNOWN;
//         let expects_inline_qos = false;
//         let proxy = RtpsReaderProxy::new(
//             remote_reader_guid,
//             remote_group_entity_id,
//             &[Locator::new(
//                 LocatorKind::new(1),
//                 LocatorPort::new(2),
//                 LocatorAddress::new([3; 16]),
//             )],
//             &[],
//             expects_inline_qos,
//             true,
//         );
//         rtps_writer.matched_reader_add(proxy);

//         let mut transport = MockTransport::new();
//         transport
//             .expect_write()
//             .once()
//             .withf(move |message, _destination_locator| {
//                 let data_frag_submessages: Vec<_> = message
//                     .submessages
//                     .iter()
//                     .filter(|s| match s {
//                         RtpsSubmessageKind::DataFrag(_) => true,
//                         _ => false,
//                     })
//                     .collect();
//                 if data_frag_submessages.len() != 2 {
//                     false
//                 } else {
//                     let ret1 = if let RtpsSubmessageKind::DataFrag(submessage) =
//                         data_frag_submessages[0]
//                     {
//                         submessage.fragment_starting_num == FragmentNumber::new(1)
//                             && submessage.fragment_size == UShort::new(max_bytes as u16)
//                             && <&[u8]>::from(&submessage.serialized_payload).len() == max_bytes
//                     } else {
//                         false
//                     };
//                     let ret2 = if let RtpsSubmessageKind::DataFrag(submessage) =
//                         data_frag_submessages[1]
//                     {
//                         submessage.fragment_starting_num == FragmentNumber::new(2)
//                             && submessage.fragment_size
//                                 == UShort::new((data_length - max_bytes) as u16)
//                             && <&[u8]>::from(&submessage.serialized_payload).len()
//                                 == data_length - max_bytes
//                     } else {
//                         false
//                     };
//                     ret1 && ret2
//                 }
//             })
//             .return_const(());

//         rtps_writer.send_submessage_reliable(&mut transport);
//     }
// }
