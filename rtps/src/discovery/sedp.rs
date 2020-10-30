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
use crate::behavior::types::Duration;

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
        let sedp_builtin_publications_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let sedp_builtin_publications_writer_qos = DataWriterQos::default(); // TODO
        let sedp_builtin_publications_writer = StatefulWriter::new(
            sedp_builtin_publications_writer_guid,
            TopicKind::WithKey,
            &sedp_builtin_publications_writer_qos
        ); 

        let sedp_builtin_publications_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_reader_qos = DataReaderQos::default(); // TODO
        let sedp_builtin_publications_reader = StatefulReader::new(
            sedp_builtin_publications_reader_guid,
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            false,
            Duration::from_millis(500),
            &sedp_builtin_publications_reader_qos
        );

        let sedp_builtin_subscriptions_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let sedp_builtin_subscriptions_writer_qos = DataWriterQos::default(); // TODO
        let sedp_builtin_subscriptions_writer = StatefulWriter::new(
            sedp_builtin_subscriptions_writer_guid,
            TopicKind::WithKey,
            &sedp_builtin_subscriptions_writer_qos
        );

        let sedp_builtin_subscriptions_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_reader_qos = DataReaderQos::default(); // TODO
        let sedp_builtin_subscriptions_reader = StatefulReader::new(
            sedp_builtin_subscriptions_reader_guid,
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            false,
            Duration::from_millis(500),
            &sedp_builtin_subscriptions_reader_qos
        );

        let sedp_builtin_topics_writer_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let sedp_builtin_topics_writer_qos = DataWriterQos::default(); // TODO
        let sedp_builtin_topics_writer = StatefulWriter::new(
            sedp_builtin_topics_writer_guid,
            TopicKind::WithKey,
            &sedp_builtin_topics_writer_qos
        );

        let sedp_builtin_topics_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_reader_qos = DataReaderQos::default(); // TODO
        let sedp_builtin_topics_reader = StatefulReader::new(
            sedp_builtin_topics_reader_guid,
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            false,
            Duration::from_millis(500),
            &sedp_builtin_topics_reader_qos
        );


        Self {
            sedp_builtin_publications_writer,
            sedp_builtin_publications_reader,
            sedp_builtin_subscriptions_writer,
            sedp_builtin_subscriptions_reader,
            sedp_builtin_topics_writer,
            sedp_builtin_topics_reader,
        }
    }
}
