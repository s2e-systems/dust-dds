use dds_transport::{
    messages::submessages::{AckNackSubmessage, GapSubmessage, HeartbeatSubmessage},
    types::Locator,
};

use crate::dcps_psm::Duration;

use super::{
    endpoint::RtpsEndpointImpl,
    history_cache::RtpsHistoryCacheImpl,
    reader::RtpsReaderImpl,
    types::{Count, Guid, GuidPrefix, ReliabilityKind, TopicKind},
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

pub struct RtpsStatefulReaderImpl {
    reader: RtpsReaderImpl,
    matched_writers: Vec<RtpsWriterProxy>,
}

impl RtpsStatefulReaderImpl {
    pub fn process_gap_submessage(
        &mut self,
        gap_submessage: &GapSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, gap_submessage.writer_id.value.into());
        let reliability_level = self.reliability_level();
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == writer_guid)
        {
            match reliability_level {
                ReliabilityKind::BestEffort => writer_proxy.best_effort_receive_gap(gap_submessage),
                ReliabilityKind::Reliable => writer_proxy.reliable_receive_gap(gap_submessage),
            }
        }
    }
}

impl RtpsStatefulReaderImpl {
    pub fn guid(&self) -> Guid {
        self.reader.guid()
    }
}

impl RtpsStatefulReaderImpl {
    pub fn topic_kind(&self) -> TopicKind {
        self.reader.topic_kind()
    }

    pub fn reliability_level(&self) -> ReliabilityKind {
        self.reader.reliability_level()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.reader.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.reader.multicast_locator_list()
    }
}

impl RtpsStatefulReaderImpl {
    pub fn heartbeat_response_delay(&self) -> Duration {
        self.reader.heartbeat_response_delay()
    }

    pub fn heartbeat_suppression_duration(&self) -> Duration {
        self.reader.heartbeat_suppression_duration()
    }

    pub fn reader_cache(&mut self) -> &mut RtpsHistoryCacheImpl {
        self.reader.reader_cache()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.reader.expects_inline_qos()
    }
}

impl RtpsStatefulReaderImpl {
    pub fn matched_writers(&mut self) -> &mut [RtpsWriterProxy] {
        self.matched_writers.as_mut_slice()
    }
}

impl RtpsStatefulReaderImpl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            reader: RtpsReaderImpl::new(
                RtpsEndpointImpl::new(
                    guid,
                    topic_kind,
                    reliability_level,
                    unicast_locator_list,
                    multicast_locator_list,
                ),
                heartbeat_response_delay,
                heartbeat_suppression_duration,
                expects_inline_qos,
            ),
            matched_writers: Vec::new(),
        }
    }
}

impl RtpsStatefulReaderImpl {
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

impl RtpsStatefulReaderImpl {
    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.reliability_level() == ReliabilityKind::Reliable {
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

impl RtpsStatefulReaderImpl {
    pub fn send_submessages(
        &mut self,
        mut send_acknack: impl FnMut(&RtpsWriterProxy, AckNackSubmessage),
    ) {
        let entity_id = self.guid().entity_id;
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
