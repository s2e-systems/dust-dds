use super::{
    error::RtpsResult, message_receiver::MessageReceiver, message_sender::WriteMessage,
    writer_proxy::RtpsWriterProxy,
};
use crate::{
    rtps_messages::{
        self,
        overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
        submessages::{
            data::DataSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
            heartbeat::HeartbeatSubmessage, heartbeat_frag::HeartbeatFragSubmessage,
        },
    },
    transport::{
        history_cache::{CacheChange, HistoryCache},
        reader::WriterProxy,
        types::{Guid, GuidPrefix, ReliabilityKind},
    },
};
use alloc::{boxed::Box, vec::Vec};

pub struct RtpsStatefulReader {
    guid: Guid,
    matched_writers: Vec<RtpsWriterProxy>,
    history_cache: Box<dyn HistoryCache>,
}

impl RtpsStatefulReader {
    pub fn new(guid: Guid, history_cache: Box<dyn HistoryCache>) -> Self {
        Self {
            guid,
            matched_writers: Vec::new(),
            history_cache,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn add_matched_writer(&mut self, writer_proxy: &WriterProxy) {
        if self
            .matched_writers
            .iter()
            .any(|wp| wp.remote_writer_guid() == writer_proxy.remote_writer_guid)
        {
            return;
        }

        let rtps_writer_proxy = RtpsWriterProxy::new(
            writer_proxy.remote_writer_guid,
            &writer_proxy.unicast_locator_list,
            &writer_proxy.multicast_locator_list,
            writer_proxy.remote_group_entity_id,
            writer_proxy.reliability_kind,
        );
        self.matched_writers.push(rtps_writer_proxy);
    }

    pub fn delete_matched_writer(&mut self, writer_guid: Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != writer_guid)
    }

    pub fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut RtpsWriterProxy> {
        self.matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == a_writer_guid)
    }

    pub async fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<rtps_messages::types::Time>,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id());
        let sequence_number = data_submessage.writer_sn();
        if let Some(writer_proxy) = self.matched_writer_lookup(writer_guid) {
            match writer_proxy.reliability() {
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
                            self.history_cache.add_change(change).await;
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
                            self.history_cache.add_change(change).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<rtps_messages::types::Time>,
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
                )
                .await;
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

    pub async fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
        message_writer: &impl WriteMessage,
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
                writer_proxy.write_message(&self.guid, message_writer).await;
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

    pub async fn process_message(
        &mut self,
        datagram: &[u8],
        message_writer: &impl WriteMessage,
    ) -> RtpsResult<()> {
        let rtps_message = RtpsMessageRead::try_from(datagram)?;
        let mut message_receiver = MessageReceiver::new(&rtps_message);

        while let Some(submessage) = message_receiver.next() {
            match submessage {
                RtpsSubmessageReadKind::Data(data_submessage) => {
                    self.on_data_submessage_received(
                        data_submessage,
                        message_receiver.source_guid_prefix(),
                        message_receiver.source_timestamp(),
                    )
                    .await;
                }
                RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
                    self.on_data_frag_submessage_received(
                        data_frag_submessage,
                        message_receiver.source_guid_prefix(),
                        message_receiver.source_timestamp(),
                    )
                    .await;
                }
                RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => {
                    self.on_heartbeat_frag_submessage_received(
                        heartbeat_frag_submessage,
                        message_receiver.source_guid_prefix(),
                    );
                }
                RtpsSubmessageReadKind::Gap(gap_submessage) => {
                    self.on_gap_submessage_received(
                        gap_submessage,
                        message_receiver.source_guid_prefix(),
                    );
                }
                RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => {
                    self.on_heartbeat_submessage_received(
                        heartbeat_submessage,
                        message_receiver.source_guid_prefix(),
                        message_writer,
                    )
                    .await;
                }
                _ => (),
            }
        }
        Ok(())
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
}
