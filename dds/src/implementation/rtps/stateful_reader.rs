use dds_transport::messages::submessages::{AckNackSubmessage, GapSubmessage, HeartbeatSubmessage};

use crate::infrastructure::qos_policy::ReliabilityQosPolicyKind;

use super::{
    reader::RtpsReader,
    types::{Count, Guid, GuidPrefix},
    writer_proxy::RtpsWriterProxy,
};

/// ChangeFromWriterStatusKind
/// Enumeration used to indicate the status of a ChangeFromWriter. It can take the values:
/// LOST, MISSING, RECEIVED, UNKNOWN
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
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
    pub fn reader(&self) -> &RtpsReader {
        &self.reader
    }

    pub fn reader_mut(&mut self) -> &mut RtpsReader {
        &mut self.reader
    }

    pub fn matched_writers(&mut self) -> &mut [RtpsWriterProxy] {
        self.matched_writers.as_mut_slice()
    }
}

impl RtpsStatefulReader {
    pub fn new(reader: RtpsReader) -> Self {
        Self {
            reader,
            matched_writers: Vec::new(),
        }
    }
}

impl RtpsStatefulReader {
    pub fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy) {
        self.matched_writers.push(a_writer_proxy);
    }

    pub fn matched_writer_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsWriterProxy) -> bool,
    {
        self.matched_writers.retain(|x| !f(x))
    }

    pub fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut RtpsWriterProxy> {
        self.matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == a_writer_guid)
    }
}

impl RtpsStatefulReader {
    pub fn process_gap_submessage(
        &mut self,
        gap_submessage: &GapSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, gap_submessage.writer_id.value.into());
        let reliability_level = &self.reader.get_qos().reliability.kind;
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == writer_guid)
        {
            match reliability_level {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => {
                    writer_proxy.best_effort_receive_gap(gap_submessage)
                }
                ReliabilityQosPolicyKind::ReliableReliabilityQos => {
                    writer_proxy.reliable_receive_gap(gap_submessage)
                }
            }
        }
    }
}

impl RtpsStatefulReader {
    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.reader.get_qos().reliability.kind
            == ReliabilityQosPolicyKind::ReliableReliabilityQos
        {
            let writer_guid = Guid::new(
                source_guid_prefix,
                heartbeat_submessage.writer_id.value.into(),
            );

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count.0 != heartbeat_submessage.count.value
                {
                    writer_proxy.last_received_heartbeat_count.0 = heartbeat_submessage.count.value;

                    writer_proxy.must_send_acknacks = !heartbeat_submessage.final_flag
                        || (!heartbeat_submessage.liveliness_flag
                            && !writer_proxy.missing_changes().is_empty());

                    writer_proxy.reliable_receive_heartbeat(heartbeat_submessage);
                }
            }
        }
    }
}

impl RtpsStatefulReader {
    pub fn send_submessages(
        &mut self,
        mut send_acknack: impl FnMut(&RtpsWriterProxy, AckNackSubmessage),
    ) {
        let entity_id = self.reader.guid().entity_id;
        for writer_proxy in self.matched_writers.iter_mut() {
            if writer_proxy.must_send_acknacks {
                if !writer_proxy.missing_changes().is_empty() {
                    writer_proxy.acknack_count =
                        Count(writer_proxy.acknack_count.0.wrapping_add(1));

                    writer_proxy.reliable_send_ack_nack(
                        entity_id,
                        writer_proxy.acknack_count,
                        |wp, acknack| send_acknack(wp, acknack),
                    );
                }
                writer_proxy.must_send_acknacks = false;
            }
        }
    }
}
