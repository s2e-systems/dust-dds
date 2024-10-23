use crate::{
    builtin_topics::ParticipantBuiltinTopicData,
    implementation::data_representation_builtin_endpoints::{
        discovered_reader_data::ReaderProxy,
        discovered_writer_data::WriterProxy,
        spdp_discovered_participant_data::{ParticipantProxy, SpdpDiscoveredParticipantData},
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize},
};

use super::{
    behavior_types::Duration,
    cache_change::RtpsCacheChange,
    message_sender::MessageSender,
    messages::{
        submessage_elements::SequenceNumberSet,
        submessages::{gap::GapSubmessage, info_timestamp::InfoTimestampSubmessage},
    },
    reader_locator::RtpsReaderLocator,
    stateful_writer::{TransportWriter, WriterHistoryCache},
    types::{DurabilityKind, Guid, Locator, ReliabilityKind, SequenceNumber, ENTITYID_UNKNOWN},
};

pub struct RtpsStatelessWriter {
    guid: Guid,
    changes: Vec<RtpsCacheChange>,
    reader_locators: Vec<RtpsReaderLocator>,
    message_sender: MessageSender,
    participant_proxy: ParticipantProxy,
}

impl RtpsStatelessWriter {
    pub fn new(
        guid: Guid,
        message_sender: MessageSender,
        participant_proxy: ParticipantProxy,
    ) -> Self {
        Self {
            guid,
            changes: Vec::new(),
            reader_locators: Vec::new(),
            message_sender,
            participant_proxy,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn reader_locator_add(&mut self, locator: Locator) {
        self.reader_locators
            .push(RtpsReaderLocator::new(locator, false));
    }

    pub fn send_message(&mut self) {
        for reader_locator in &mut self.reader_locators {
            while let Some(unsent_change_seq_num) =
                reader_locator.next_unsent_change(self.changes.iter())
            {
                // The post-condition:
                // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
                // should be full-filled by next_unsent_change()

                if let Some(cache_change) = self
                    .changes
                    .iter()
                    .find(|cc| cc.sequence_number() == unsent_change_seq_num)
                {
                    if let Some(timestamp) = cache_change.source_timestamp() {
                        let info_ts_submessage =
                            Box::new(InfoTimestampSubmessage::new(false, timestamp));
                        let data_submessage = Box::new(
                            cache_change
                                .as_data_submessage(ENTITYID_UNKNOWN, self.guid.entity_id()),
                        );

                        self.message_sender.write_message(
                            &[info_ts_submessage, data_submessage],
                            vec![reader_locator.locator()],
                        );
                    }
                } else {
                    let gap_submessage = Box::new(GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        self.guid.entity_id(),
                        unsent_change_seq_num,
                        SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                    ));

                    self.message_sender
                        .write_message(&[gap_submessage], vec![reader_locator.locator()]);
                }
                reader_locator.set_highest_sent_change_sn(unsent_change_seq_num);
            }
        }
    }
}

impl TransportWriter for RtpsStatelessWriter {
    fn get_history_cache(&mut self) -> &mut dyn WriterHistoryCache {
        self
    }

    fn add_matched_reader(
        &mut self,
        reader_proxy: ReaderProxy,
        _reliability_kind: ReliabilityKind,
        _durability_kind: DurabilityKind,
    ) {
        for unicast_locator in reader_proxy.unicast_locator_list {
            self.reader_locator_add(unicast_locator);
        }
        for multicast_locator in reader_proxy.multicast_locator_list {
            self.reader_locator_add(multicast_locator);
        }
    }

    fn delete_matched_reader(&mut self, _reader_guid: Guid) {
        // Do nothing
    }

    fn are_all_changes_acknowledged(&self) -> bool {
        true
    }

    fn writer_proxy(&self) -> WriterProxy {
        WriterProxy {
            remote_writer_guid: self.guid,
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            data_max_size_serialized: Default::default(),
        }
    }
}

impl WriterHistoryCache for RtpsStatelessWriter {
    fn add_change(&mut self, mut cache_change: RtpsCacheChange) {
        let dds_participant_data =
            ParticipantBuiltinTopicData::deserialize_data(cache_change.data_value.as_ref())
                .unwrap();
        let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
            dds_participant_data,
            participant_proxy: self.participant_proxy.clone(),
            lease_duration: Duration::new(100, 0).into(),
            discovered_participant_list: vec![],
        };

        cache_change.data_value = spdp_discovered_participant_data
            .serialize_data()
            .unwrap()
            .into();
        self.changes.push(cache_change);
        self.send_message();
    }

    fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.changes
            .retain(|cc| cc.sequence_number() != sequence_number);
    }
}
