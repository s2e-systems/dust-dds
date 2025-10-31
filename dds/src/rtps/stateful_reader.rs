use super::{error::RtpsResult, message_receiver::MessageReceiver, writer_proxy::RtpsWriterProxy};
use crate::{
    rtps::message_sender::WriteMessage,
    rtps_messages::{
        self,
        overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
        submessages::{
            data::DataSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
            heartbeat::HeartbeatSubmessage, heartbeat_frag::HeartbeatFragSubmessage,
        },
    },
    transport::{
        interface::HistoryCache,
        types::{CacheChange, Guid, GuidPrefix, ReliabilityKind, WriterProxy},
    },
};
use alloc::{boxed::Box, vec::Vec};

pub struct RtpsStatefulReader {
    guid: Guid,
    matched_writers: Vec<RtpsWriterProxy>,
    reliability: ReliabilityKind,
    history_cache: Box<dyn HistoryCache>,
}

impl RtpsStatefulReader {
    pub fn new(
        guid: Guid,
        history_cache: Box<dyn HistoryCache>,
        reliability: ReliabilityKind,
    ) -> Self {
        Self {
            guid,
            matched_writers: Vec::new(),
            history_cache,
            reliability,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn add_matched_writer(&mut self, writer_proxy: &WriterProxy) {
        let rtps_writer_proxy = RtpsWriterProxy::new(
            writer_proxy.remote_writer_guid,
            &writer_proxy.unicast_locator_list,
            &writer_proxy.multicast_locator_list,
            writer_proxy.remote_group_entity_id,
            writer_proxy.reliability_kind,
        );
        if let Some(wp) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_proxy.remote_writer_guid)
        {
            *wp = rtps_writer_proxy;
        } else {
            self.matched_writers.push(rtps_writer_proxy);
        }
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
        let reliability = self.reliability;
        if let Some(writer_proxy) = self.matched_writer_lookup(writer_guid) {
            match reliability {
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
            match writer_proxy.reliability() {
                ReliabilityKind::BestEffort => {
                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                    if sequence_number >= expected_seq_num {
                        writer_proxy.push_data_frag(data_frag_submessage.clone());
                    }
                }
                ReliabilityKind::Reliable => {
                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                    if sequence_number == expected_seq_num {
                        writer_proxy.push_data_frag(data_frag_submessage.clone());
                    }
                }
            }

            if let Some(data_submessage) = writer_proxy.reconstruct_data_from_frag(sequence_number)
            {
                writer_proxy.delete_data_fragments(data_submessage.writer_sn());
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
                writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());

                let must_send_acknacks = !heartbeat_submessage.final_flag()
                    || (!heartbeat_submessage.liveliness_flag()
                        && writer_proxy.missing_changes().count() > 0);
                writer_proxy.set_must_send_acknacks(must_send_acknacks);

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
        rtps_message: &RtpsMessageRead,
        message_writer: &impl WriteMessage,
    ) -> RtpsResult<()> {
        let mut message_receiver = MessageReceiver::new(rtps_message);

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

#[cfg(test)]
mod tests {
    use core::panic;
    use std::sync::Arc;

    use crate::{
        rtps_messages::submessage_elements::{Data, ParameterList, SerializedDataFragment},
        std_runtime::executor::block_on,
        transport::types::{DurabilityKind, ENTITYID_UNKNOWN, EntityId},
    };

    use super::*;

    #[test]
    fn receive_only_not_yet_received_data_frag() {
        struct MockCache;

        impl HistoryCache for MockCache {
            fn add_change(
                &mut self,
                _cache_change: CacheChange,
            ) -> core::pin::Pin<Box<dyn Future<Output = ()> + Send + '_>> {
                todo!()
            }

            fn remove_change(
                &mut self,
                _sequence_number: i64,
            ) -> core::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
                todo!()
            }
        }

        struct MockWriter;
        impl WriteMessage for MockWriter {
            async fn write_message(
                &self,
                datagram: &[u8],
                _locator_list: &[crate::transport::types::Locator],
            ) {
                let message = RtpsMessageRead::try_from(datagram).unwrap();
                let submessages = message.submessages();
                match &submessages[1] {
                    RtpsSubmessageReadKind::AckNack(ack_nack_submessage) => {
                        assert_eq!(ack_nack_submessage.reader_sn_state().base(), 1);
                        assert_eq!(ack_nack_submessage.reader_sn_state().set().count(), 0);
                    }
                    _ => panic!("Expected AckNack submessage"),
                }
                match &submessages[2] {
                    RtpsSubmessageReadKind::NackFrag(nack_frag_submessage) => {
                        assert_eq!(nack_frag_submessage.writer_sn(), 1);
                        assert_eq!(nack_frag_submessage.fragment_number_state().base(), 2);
                        assert_eq!(
                            nack_frag_submessage
                                .fragment_number_state()
                                .set()
                                .collect::<Vec<u32>>(),
                            vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
                        );
                    }
                    _ => panic!("Expected NackFrag submessage"),
                }
            }

            fn guid_prefix(&self) -> GuidPrefix {
                [2; 12]
            }
        }

        let reader_guid_prefix = [1; 12];
        let reader_entity_id = EntityId::new([1; 3], 1);
        let reader_guid = Guid::new(reader_guid_prefix, reader_entity_id);
        let mut reader =
            RtpsStatefulReader::new(reader_guid, Box::new(MockCache), ReliabilityKind::Reliable);

        let writer_guid_prefix = [2; 12];
        let writer_entity_id = EntityId::new([2; 3], 2);
        reader.add_matched_writer(&WriterProxy {
            remote_writer_guid: Guid::new(writer_guid_prefix, writer_entity_id),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
        });

        let writer_sn = 1;

        let payload = Arc::<[u8]>::from(vec![1; 1200]);
        let data_frag1_submessage = DataFragSubmessage::new(
            false,
            false,
            false,
            reader_entity_id,
            writer_entity_id,
            writer_sn,
            1,
            1,
            100,
            payload.len() as u32,
            ParameterList::empty(),
            SerializedDataFragment::new(Data::new(payload), 0..100),
        );
        block_on(reader.on_data_frag_submessage_received(
            &data_frag1_submessage,
            writer_guid_prefix,
            None,
        ));

        let heartbeat_submessage =
            HeartbeatSubmessage::new(false, false, reader_entity_id, writer_entity_id, 1, 1, 1);
        let message_writer = MockWriter;
        block_on(reader.on_heartbeat_submessage_received(
            &heartbeat_submessage,
            writer_guid_prefix,
            &message_writer,
        ))
    }
}
