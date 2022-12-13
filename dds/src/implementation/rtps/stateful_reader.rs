use crate::{
    implementation::dds_impl::message_receiver::MessageReceiver,
    infrastructure::{
        qos_policy::ReliabilityQosPolicyKind,
        time::{Duration, DURATION_ZERO},
    },
};

use super::{
    messages::{
        overall_structure::RtpsMessageHeader,
        submessage_elements::{
            GuidPrefixSubmessageElement, ProtocolVersionSubmessageElement,
            VendorIdSubmessageElement,
        },
        submessages::{AckNackSubmessage, DataSubmessage, HeartbeatSubmessage},
        types::ProtocolId,
        RtpsMessage, RtpsSubmessageKind,
    },
    reader::RtpsReader,
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
                if writer_proxy.last_received_heartbeat_count != heartbeat_submessage.count.value {
                    writer_proxy.last_received_heartbeat_count = heartbeat_submessage.count.value;

                    writer_proxy.must_send_acknacks = !heartbeat_submessage.final_flag
                        || (!heartbeat_submessage.liveliness_flag
                            && !writer_proxy.missing_changes().is_empty());

                    if !heartbeat_submessage.final_flag {
                        writer_proxy.must_send_acknacks = true;
                    }
                    writer_proxy.missing_changes_update(heartbeat_submessage.last_sn.value);
                    writer_proxy.lost_changes_update(heartbeat_submessage.first_sn.value);
                }
            }
        }
    }

    pub fn send_submessages(
        &mut self,
        mut send_acknack: impl FnMut(&RtpsWriterProxy, AckNackSubmessage),
    ) {
        let entity_id = self.reader.guid().entity_id();
        for writer_proxy in self.matched_writers.iter_mut() {
            if writer_proxy.must_send_acknacks || !writer_proxy.missing_changes().is_empty() {
                writer_proxy.acknack_count = writer_proxy.acknack_count.wrapping_add(1);

                writer_proxy.reliable_send_ack_nack(
                    entity_id,
                    writer_proxy.acknack_count,
                    |wp, acknack| send_acknack(wp, acknack),
                );
                writer_proxy.must_send_acknacks = false;
            }
        }
    }

    pub fn send_message(&mut self, transport: &mut impl TransportWrite) {
        let mut acknacks = Vec::new();
        self.send_submessages(|wp, acknack| {
            acknacks.push((
                wp.unicast_locator_list().to_vec(),
                vec![RtpsSubmessageKind::AckNack(acknack)],
            ))
        });

        for (locator_list, acknacks) in acknacks {
            let header = RtpsMessageHeader {
                protocol: ProtocolId::PROTOCOL_RTPS,
                version: ProtocolVersionSubmessageElement {
                    value: PROTOCOLVERSION,
                },
                vendor_id: VendorIdSubmessageElement {
                    value: VENDOR_ID_S2E,
                },
                guid_prefix: GuidPrefixSubmessageElement {
                    value: self.reader().guid().prefix(),
                },
            };

            let message = RtpsMessage {
                header,
                submessages: acknacks,
            };

            for &locator in &locator_list {
                transport.write(&message, locator);
            }
        }
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
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
                        writer_proxy.received_change_set(sequence_number);
                        if sequence_number > expected_seq_num {
                            writer_proxy.lost_changes_update(sequence_number);
                        }

                        self.reader
                            .add_change(
                                data_submessage,
                                Some(message_receiver.timestamp()),
                                message_receiver.source_guid_prefix(),
                                message_receiver.reception_timestamp(),
                            )
                            .ok();
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    if sequence_number == expected_seq_num {
                        writer_proxy.received_change_set(data_submessage.writer_sn.value);

                        self.reader
                            .add_change(
                                data_submessage,
                                Some(message_receiver.timestamp()),
                                message_receiver.source_guid_prefix(),
                                message_receiver.reception_timestamp(),
                            )
                            .ok();
                    }
                }
            }
        }
    }
}
