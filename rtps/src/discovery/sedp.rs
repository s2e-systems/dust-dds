use std::sync::{Arc, Mutex};
use crate::types::{GUID, GuidPrefix, TopicKind, ReliabilityKind};
use crate::types::constants::{
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
};
use crate::behavior::{StatefulReader, StatefulWriter};
use crate::behavior::stateful_reader::NoOpStatefulReaderListener;
use crate::behavior::types::Duration;
use crate::structure::HistoryCacheResourceLimits;

#[derive(Clone)]
pub struct SimpleEndpointDiscoveryProtocol {
    sedp_builtin_publications_writer: Arc<Mutex<StatefulWriter>>,
    sedp_builtin_publications_reader: Arc<Mutex<StatefulReader>>,
    sedp_builtin_subscriptions_writer: Arc<Mutex<StatefulWriter>>,
    sedp_builtin_subscriptions_reader: Arc<Mutex<StatefulReader>>,
    sedp_builtin_topics_writer: Arc<Mutex<StatefulWriter>>,
    sedp_builtin_topics_reader: Arc<Mutex<StatefulReader>>,
}

impl SimpleEndpointDiscoveryProtocol {
    pub fn new(guid_prefix: GuidPrefix) -> Self {

        let resource_limits = HistoryCacheResourceLimits::default();
        let reliability_level = ReliabilityKind::Reliable;
        let heartbeat_period = Duration::from_millis(100);
        let nack_response_delay = Duration::from_millis(100);
        let nack_suppression_duration = Duration::from_millis(100);

        let sedp_builtin_publications_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let sedp_builtin_publications_writer = Arc::new(Mutex::new(StatefulWriter::new(
            sedp_builtin_publications_writer_guid,
            TopicKind::WithKey,
            reliability_level,
            resource_limits,
            true,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration
        )));

        let heartbeat_response_delay = Duration::from_millis(500);

        let sedp_builtin_publications_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_reader = Arc::new(Mutex::new(StatefulReader::new(
            sedp_builtin_publications_reader_guid,
            TopicKind::WithKey,
            reliability_level,
            false,
            heartbeat_response_delay,
            resource_limits,
            NoOpStatefulReaderListener,
        )));

        let sedp_builtin_subscriptions_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let sedp_builtin_subscriptions_writer = Arc::new(Mutex::new(StatefulWriter::new(
            sedp_builtin_subscriptions_writer_guid,
            TopicKind::WithKey,
            reliability_level,
            resource_limits,
            true,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration
        )));

        let sedp_builtin_subscriptions_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_reader = Arc::new(Mutex::new(StatefulReader::new(
            sedp_builtin_subscriptions_reader_guid,
            TopicKind::WithKey,
            reliability_level,
            false,
            heartbeat_response_delay,
            resource_limits,
            NoOpStatefulReaderListener,
        )));

        let sedp_builtin_topics_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let sedp_builtin_topics_writer = Arc::new(Mutex::new(StatefulWriter::new(
            sedp_builtin_topics_writer_guid,
            TopicKind::WithKey,
            reliability_level,
            resource_limits,
            true,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration
        )));

        let sedp_builtin_topics_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_reader = Arc::new(Mutex::new(StatefulReader::new(
            sedp_builtin_topics_reader_guid,
            TopicKind::WithKey,
            reliability_level,
            false,
            heartbeat_response_delay,
            resource_limits,
            NoOpStatefulReaderListener,
        )));


        Self {
            sedp_builtin_publications_writer,
            sedp_builtin_publications_reader,
            sedp_builtin_subscriptions_writer,
            sedp_builtin_subscriptions_reader,
            sedp_builtin_topics_writer,
            sedp_builtin_topics_reader,
        }
    }

    pub fn sedp_builtin_publications_writer(&self) -> &Arc<Mutex<StatefulWriter>> {
        &self.sedp_builtin_publications_writer
    }
    pub fn sedp_builtin_publications_reader(&self) -> &Arc<Mutex<StatefulReader>> {
        &self.sedp_builtin_publications_reader
    }
    pub fn sedp_builtin_subscriptions_writer(&self) -> &Arc<Mutex<StatefulWriter>> {
        &self.sedp_builtin_subscriptions_writer
    }
    pub fn sedp_builtin_subscriptions_reader(&self) -> &Arc<Mutex<StatefulReader>> {
        &self.sedp_builtin_subscriptions_reader
    }
    pub fn sedp_builtin_topics_writer(&self) -> &Arc<Mutex<StatefulWriter>> {
        &self.sedp_builtin_topics_writer
    }
    pub fn sedp_builtin_topics_reader(&self) -> &Arc<Mutex<StatefulReader>> {
        &self.sedp_builtin_topics_reader
    }

}