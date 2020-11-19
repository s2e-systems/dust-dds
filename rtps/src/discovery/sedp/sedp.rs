use crate::types::{GUID, GuidPrefix, ReliabilityKind, EntityId};
use crate::types::constants::{
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
};
use crate::structure::{RtpsEntity, RtpsEndpoint};
use crate::behavior::{StatefulReader, StatefulWriter, RtpsWriter, RtpsReader};
use crate::behavior::stateful_reader::NoOpStatefulReaderListener;
use crate::behavior::types::Duration;

use rust_dds_interface::types::TopicKind;
use rust_dds_interface::history_cache::HistoryCache;

pub struct SimpleEndpointDiscoveryProtocol {
    sedp_builtin_publications_writer: StatefulWriter,
    sedp_builtin_publications_reader: StatefulReader,
    sedp_builtin_subscriptions_writer: StatefulWriter,
    sedp_builtin_subscriptions_reader: StatefulReader,
    sedp_builtin_topics_writer: StatefulWriter,
    sedp_builtin_topics_reader: StatefulReader,
}

impl SimpleEndpointDiscoveryProtocol {
    fn endpoint(guid_prefix: GuidPrefix, entity_id: EntityId) -> RtpsEndpoint {
        let guid = GUID::new(guid_prefix, entity_id);
        let entity = RtpsEntity::new(guid);
        
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        RtpsEndpoint::new(entity, topic_kind, reliability_level)
    }

    fn reader(guid_prefix: GuidPrefix, entity_id: EntityId) ->  RtpsReader {
        let expects_inline_qos = false;
        RtpsReader::new(Self::endpoint(guid_prefix, entity_id), HistoryCache::default(), expects_inline_qos)
    }

    fn writer(guid_prefix: GuidPrefix, entity_id: EntityId) ->  RtpsWriter {
        let push_mode = true;
        let data_max_sized_serialized = None;
        RtpsWriter::new(Self::endpoint(guid_prefix, entity_id), push_mode, HistoryCache::default(), data_max_sized_serialized)
    }

    fn stateful_reader(guid_prefix: GuidPrefix, entity_id: EntityId) -> StatefulReader {
        let heartbeat_response_delay = Duration::from_millis(500);     
        StatefulReader::new(Self::reader(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR), heartbeat_response_delay)
    }

    fn stateful_writer(guid_prefix: GuidPrefix, entity_id: EntityId) -> StatefulWriter {
        let heartbeat_period = Duration::from_millis(100);
        let nack_response_delay = Duration::from_millis(100);
        let nack_suppression_duration = Duration::from_millis(100);
        StatefulWriter::new(
            Self::writer(guid_prefix, entity_id),
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration
        )
    }

    pub fn new(guid_prefix: GuidPrefix) -> Self {
        
        let sedp_builtin_publications_writer = Self::stateful_writer(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);      
        let sedp_builtin_publications_reader = Self::stateful_reader(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR); 
        let sedp_builtin_subscriptions_writer = Self::stateful_writer(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);  
        let sedp_builtin_subscriptions_reader = Self::stateful_reader(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR); 
        let sedp_builtin_topics_writer = Self::stateful_writer(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let sedp_builtin_topics_reader = Self::stateful_reader(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);

        Self {
            sedp_builtin_publications_writer,
            sedp_builtin_publications_reader,
            sedp_builtin_subscriptions_writer,
            sedp_builtin_subscriptions_reader,
            sedp_builtin_topics_writer,
            sedp_builtin_topics_reader,
        }
    }

    pub fn sedp_builtin_publications_writer(&mut self) -> &mut StatefulWriter {
        &mut self.sedp_builtin_publications_writer
    }
    pub fn sedp_builtin_publications_reader(&mut self) -> &mut StatefulReader {
        &mut self.sedp_builtin_publications_reader
    }
    pub fn sedp_builtin_subscriptions_writer(&mut self) -> &mut StatefulWriter {
        &mut self.sedp_builtin_subscriptions_writer
    }
    pub fn sedp_builtin_subscriptions_reader(&mut self) -> &mut StatefulReader {
        &mut self.sedp_builtin_subscriptions_reader
    }
    pub fn sedp_builtin_topics_writer(&mut self) -> &mut StatefulWriter {
        &mut self.sedp_builtin_topics_writer
    }
    pub fn sedp_builtin_topics_reader(&mut self) -> &mut StatefulReader {
        &mut self.sedp_builtin_topics_reader
    }

}