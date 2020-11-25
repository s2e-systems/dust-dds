use crate::types::{GUID, GuidPrefix, ReliabilityKind,};
use crate::types::constants::{
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
};
use crate::behavior::{StatefulReader, StatefulWriter,};
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
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let expects_inline_qos = false;
        let push_mode = true;
        let data_max_sized_serialized = None;
        let heartbeat_period = Duration::from_millis(100);
        let nack_response_delay = Duration::from_millis(100);
        let nack_suppression_duration = Duration::from_millis(100);
        let heartbeat_response_delay = Duration::from_millis(500);   
        
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let writer_cache = HistoryCache::default();
        let sedp_builtin_publications_writer = StatefulWriter::new(guid, topic_kind, reliability_level, push_mode, writer_cache, data_max_sized_serialized, heartbeat_period, nack_response_delay, nack_suppression_duration);

        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let reader_cache = HistoryCache::default();
        let sedp_builtin_publications_reader = StatefulReader::new(guid, topic_kind, reliability_level, reader_cache, expects_inline_qos, heartbeat_response_delay);
        
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let writer_cache = HistoryCache::default();
        let sedp_builtin_subscriptions_writer = StatefulWriter::new(guid, topic_kind, reliability_level, push_mode, writer_cache, data_max_sized_serialized, heartbeat_period, nack_response_delay, nack_suppression_duration);
        
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_cache = HistoryCache::default();
        let sedp_builtin_subscriptions_reader = StatefulReader::new(guid, topic_kind, reliability_level, reader_cache, expects_inline_qos, heartbeat_response_delay);
        
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let writer_cache = HistoryCache::default();
        let sedp_builtin_topics_writer = StatefulWriter::new(guid, topic_kind, reliability_level, push_mode, writer_cache, data_max_sized_serialized, heartbeat_period, nack_response_delay, nack_suppression_duration);
        
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let reader_cache = HistoryCache::default();
        let sedp_builtin_topics_reader = StatefulReader::new(guid, topic_kind, reliability_level, reader_cache, expects_inline_qos, heartbeat_response_delay);

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

    pub fn writers(&mut self) -> Vec<&mut StatefulWriter> {
        vec![&mut self.sedp_builtin_publications_writer,  &mut self.sedp_builtin_subscriptions_writer, &mut self.sedp_builtin_topics_writer]
    }
}