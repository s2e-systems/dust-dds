use crate::types::{GUID, GuidPrefix, EntityId, EntityKind, ReliabilityKind};
use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER};
use crate::behavior::types::Duration;
use crate::behavior::{StatelessWriter, StatefulWriter};

use rust_dds_api::types::TopicKind;
use crate::structure::HistoryCache;

pub struct BuiltInPublisher {
    guid: GUID,
    spdp_builtin_participant_writer: StatelessWriter,
    sedp_builtin_publications_writer: StatefulWriter,
    sedp_builtin_subscriptions_writer: StatefulWriter,
    sedp_builtin_topics_writer: StatefulWriter,
}

impl BuiltInPublisher {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        // let guid = GUID::new(guid_prefix, EntityId::new([3,3,3], EntityKind::BuiltInWriterGroup));

        // let writer_guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        // let writer_cache = HistoryCache::default();
        // let spdp_builtin_participant_writer = StatelessWriter::new(
        //     writer_guid,
        //     TopicKind::WithKey,
        //     ReliabilityKind::BestEffort,
        //     writer_cache,
        //     );
        
        // let reliability_level = ReliabilityKind::Reliable;
        // let heartbeat_period = Duration::from_millis(100);
        // let nack_response_delay = Duration::from_millis(100);
        // let nack_suppression_duration = Duration::from_millis(100);

        // let sedp_builtin_publications_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        // let writer_cache = HistoryCache::default();
        // let sedp_builtin_publications_writer = StatefulWriter::new(
        //     sedp_builtin_publications_writer_guid,
        //     TopicKind::WithKey,
        //     reliability_level,
        //     writer_cache,
        //     true,
        //     heartbeat_period,
        //     nack_response_delay,
        //     nack_suppression_duration
        // );

        // let sedp_builtin_subscriptions_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        // let writer_cache = HistoryCache::default();
        // let sedp_builtin_subscriptions_writer = StatefulWriter::new(
        //     sedp_builtin_subscriptions_writer_guid,
        //     TopicKind::WithKey,
        //     reliability_level,
        //     writer_cache,
        //     true,
        //     heartbeat_period,
        //     nack_response_delay,
        //     nack_suppression_duration
        // );

        // let sedp_builtin_topics_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        // let writer_cache = HistoryCache::default();
        // let sedp_builtin_topics_writer = StatefulWriter::new(
        //     sedp_builtin_topics_writer_guid,
        //     TopicKind::WithKey,
        //     reliability_level,
        //     writer_cache,
        //     true,
        //     heartbeat_period,
        //     nack_response_delay,
        //     nack_suppression_duration
        // );

        // Self {
        //     guid,
        //     spdp_builtin_participant_writer,
        //     sedp_builtin_publications_writer,
        //     sedp_builtin_subscriptions_writer,
        //     sedp_builtin_topics_writer,
        // }
        todo!()
    }
}