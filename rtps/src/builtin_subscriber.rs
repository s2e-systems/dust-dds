use crate::types::{GUID, GuidPrefix, EntityId, EntityKind, ReliabilityKind};
use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR};
use crate::behavior::types::Duration;
use crate::behavior::{StatelessReader, StatefulReader};

use rust_dds_interface::types::TopicKind;
use rust_dds_interface::history_cache::HistoryCache;

pub struct BuiltInSubscriber {
    guid: GUID,
    spdp_builtin_participant_reader: StatelessReader,
    sedp_builtin_publications_reader: StatefulReader,
    sedp_builtin_subscriptions_reader: StatefulReader,
    sedp_builtin_topics_reader: StatefulReader,
}

impl BuiltInSubscriber {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        let guid = GUID::new(guid_prefix, EntityId::new([3,3,3], EntityKind::BuiltInReaderGroup));

        let reliability_level = ReliabilityKind::Reliable;
        let heartbeat_period = Duration::from_millis(100);
        let nack_response_delay = Duration::from_millis(100);
        let nack_suppression_duration = Duration::from_millis(100);

        let reader_guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR);
        let reader_cache = HistoryCache::default();

        let spdp_builtin_participant_reader = StatelessReader::new(
            reader_guid,
            TopicKind::WithKey, 
            ReliabilityKind::BestEffort,
            vec![],
            vec![],// metatraffic_multicast_locator_list.clone(),
            false,
            reader_cache,
        );

        let heartbeat_response_delay = Duration::from_millis(500);

        let sedp_builtin_publications_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let reader_cache = HistoryCache::default();
        let sedp_builtin_publications_reader = StatefulReader::new(
            sedp_builtin_publications_reader_guid,
            TopicKind::WithKey,
            reliability_level,
            false,
            heartbeat_response_delay,
            reader_cache
        );


        let sedp_builtin_subscriptions_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_cache = HistoryCache::default();
        let sedp_builtin_subscriptions_reader = StatefulReader::new(
            sedp_builtin_subscriptions_reader_guid,
            TopicKind::WithKey,
            reliability_level,
            false,
            heartbeat_response_delay,
            reader_cache
        );


        let sedp_builtin_topics_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let reader_cache = HistoryCache::default();
        let sedp_builtin_topics_reader = StatefulReader::new(
            sedp_builtin_topics_reader_guid,
            TopicKind::WithKey,
            reliability_level,
            false,
            heartbeat_response_delay,
            reader_cache
        );

        Self {
            guid,
            spdp_builtin_participant_reader,
            sedp_builtin_publications_reader,
            sedp_builtin_subscriptions_reader,
            sedp_builtin_topics_reader,
        }
    }
}