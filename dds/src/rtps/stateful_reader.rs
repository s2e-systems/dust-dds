use super::{
    message_sender::MessageSender,
    messages::{
        self,
        submessages::{
            data::DataSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
            heartbeat::HeartbeatSubmessage, heartbeat_frag::HeartbeatFragSubmessage,
        },
    },
    writer_proxy::RtpsWriterProxy,
};
use crate::{
    implementation::data_representation_builtin_endpoints::{
        discovered_reader_data::ReaderProxy, discovered_writer_data::WriterProxy,
    },
    transport::{
        history_cache::{CacheChange, HistoryCache},
        types::{DurabilityKind, Guid, GuidPrefix, Locator, ReliabilityKind, ENTITYID_UNKNOWN},
    },
};
use tracing::error;

pub struct RtpsStatefulReader {
    guid: Guid,
    topic_name: String,
    matched_writers: Vec<RtpsWriterProxy>,
    history_cache: Box<dyn HistoryCache>,
}

impl RtpsStatefulReader {
    pub fn new(guid: Guid, topic_name: String, history_cache: Box<dyn HistoryCache>) -> Self {
        Self {
            guid,
            topic_name,
            matched_writers: Vec::new(),
            history_cache,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn add_matched_writer(
        &mut self,
        writer_proxy: &WriterProxy,
        reliability_kind: ReliabilityKind,
        _durability_kind: DurabilityKind,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) {
        if self
            .matched_writers
            .iter()
            .any(|wp| wp.remote_writer_guid() == writer_proxy.remote_writer_guid)
        {
            return;
        }

        let unicast_locator_list = if writer_proxy.unicast_locator_list.is_empty() {
            default_unicast_locator_list
        } else {
            &writer_proxy.unicast_locator_list
        };

        let multicast_locator_list = if writer_proxy.unicast_locator_list.is_empty() {
            default_multicast_locator_list
        } else {
            &writer_proxy.multicast_locator_list
        };

        let rtps_writer_proxy = RtpsWriterProxy::new(
            writer_proxy.remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            Some(writer_proxy.data_max_size_serialized),
            writer_proxy.remote_group_entity_id,
            reliability_kind,
        );
        self.matched_writers.push(rtps_writer_proxy);
    }

    pub fn delete_matched_writer(&mut self, writer_guid: Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != writer_guid)
    }

    pub fn reader_proxy(&self) -> ReaderProxy {
        ReaderProxy {
            remote_reader_guid: self.guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        }
    }

    pub fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut RtpsWriterProxy> {
        self.matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == a_writer_guid)
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<messages::types::Time>,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id());
        let sequence_number = data_submessage.writer_sn();
        if let Some(writer_proxy) = self.matched_writer_lookup(writer_guid) {
            match writer_proxy.reliability {
                ReliabilityKind::BestEffort => {
                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                    if sequence_number >= expected_seq_num {
                        writer_proxy.received_change_set(sequence_number);
                        if sequence_number > expected_seq_num {
                            writer_proxy.lost_changes_update(sequence_number);
                        }

                        if let Ok(change) = CacheChange::try_from_data_submessage(
                            data_submessage,
                            source_guid_prefix,
                            source_timestamp,
                        ) {
                            self.history_cache.add_change(change);
                        } else {
                            error!("Error converting data submessage to reader cache change. Discarding data")
                        }
                    }
                }
                ReliabilityKind::Reliable => {
                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                    if sequence_number == expected_seq_num {
                        writer_proxy.received_change_set(sequence_number);

                        if let Ok(change) = CacheChange::try_from_data_submessage(
                            data_submessage,
                            source_guid_prefix,
                            source_timestamp,
                        ) {
                            self.history_cache.add_change(change);
                        } else {
                            error!("Error converting data submessage to reader cache change. Discarding data")
                        }
                    }
                }
            }
        }
    }

    pub fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<messages::types::Time>,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, data_frag_submessage.writer_id());
        let sequence_number = data_frag_submessage.writer_sn();
        if let Some(writer_proxy) = self.matched_writer_lookup(writer_guid) {
            writer_proxy.push_data_frag(data_frag_submessage.clone());
            if let Some(data_submessage) = writer_proxy.reconstruct_data_from_frag(sequence_number)
            {
                self.on_data_submessage_received(
                    &data_submessage,
                    source_guid_prefix,
                    source_timestamp,
                );
            }
        }
    }

    pub fn on_gap_submessage_received(
        &mut self,
        gap_submessage: &GapSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, gap_submessage.writer_id());
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|w| w.remote_writer_guid() == writer_guid)
        {
            for seq_num in gap_submessage.gap_start()..gap_submessage.gap_list().base() {
                writer_proxy.irrelevant_change_set(seq_num)
            }

            for seq_num in gap_submessage.gap_list().set() {
                writer_proxy.irrelevant_change_set(seq_num)
            }
        }
    }

    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
        message_sender: &MessageSender,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, heartbeat_submessage.writer_id());
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|w| w.remote_writer_guid() == writer_guid)
        {
            if writer_proxy.last_received_heartbeat_count() < heartbeat_submessage.count() {
                writer_proxy.set_last_received_heartbeat_count(heartbeat_submessage.count());

                writer_proxy.set_must_send_acknacks(
                    !heartbeat_submessage.final_flag()
                        || (!heartbeat_submessage.liveliness_flag()
                            && !writer_proxy.missing_changes().count() == 0),
                );

                if !heartbeat_submessage.final_flag() {
                    writer_proxy.set_must_send_acknacks(true);
                }
                writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());
                writer_proxy.send_message(&self.guid, message_sender);
            }
        }
    }

    pub fn on_heartbeat_frag_submessage_received(
        &mut self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, heartbeat_frag_submessage.writer_id());
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|w| w.remote_writer_guid() == writer_guid)
        {
            if writer_proxy.last_received_heartbeat_count() < heartbeat_frag_submessage.count() {
                writer_proxy
                    .set_last_received_heartbeat_frag_count(heartbeat_frag_submessage.count());
            }
        }
    }
}

// The methods in this impl block are not defined by the standard
impl RtpsStatefulReader {
    pub fn is_historical_data_received(&self) -> bool {
        !self
            .matched_writers
            .iter()
            .any(|p| !p.is_historical_data_received())
    }

    pub fn send_message(&mut self, message_sender: &MessageSender) {
        for writer_proxy in self.matched_writers.iter_mut() {
            writer_proxy.send_message(&self.guid, message_sender)
        }
    }
}
