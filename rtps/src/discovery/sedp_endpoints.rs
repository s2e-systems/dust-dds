use crate::{
    behavior::{
        types::{constants::DURATION_ZERO, Duration},
        StatefulReader, StatefulWriter,
    },
    types::{
        constants::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        },
        GuidPrefix, Locator, ReliabilityKind, TopicKind, GUID,
    },
};

// This file implements the endpoints described in section
// 8.5.4.2 The built-in Endpoints used by the Simple Endpoint Discovery Protocol
// of the RTPS standard

pub struct SEDPBuiltinPublicationsWriter;

impl SEDPBuiltinPublicationsWriter {
    pub fn new(
        guid_prefix: GuidPrefix,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        heartbeat_period: Duration,
    ) -> StatefulWriter {
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let push_mode = true;
        let nack_response_delay = Duration::from_millis(200);
        let nack_suppression_duration = DURATION_ZERO;
        let data_max_sized_serialized = None;

        StatefulWriter::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_sized_serialized,
        )
    }
}
pub struct SEDPBuiltinPublicationsReader;

impl SEDPBuiltinPublicationsReader {
    pub fn new(
        guid_prefix: GuidPrefix,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    ) -> StatefulReader {
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let expects_inline_qos = true;
        let heartbeat_response_delay = Duration::from_millis(500);
        let heartbeat_supression_duration = DURATION_ZERO;

        StatefulReader::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        )
    }
}

pub struct SEDPBuiltinSubscriptionsWriter;

impl SEDPBuiltinSubscriptionsWriter {
    pub fn new(
        guid_prefix: GuidPrefix,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        heartbeat_period: Duration,
    ) -> StatefulWriter {
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let push_mode = true;
        let nack_response_delay = Duration::from_millis(200);
        let nack_suppression_duration = DURATION_ZERO;
        let data_max_sized_serialized = None;

        StatefulWriter::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_sized_serialized,
        )
    }
}
pub struct SEDPBuiltinSubscriptionsReader;

impl SEDPBuiltinSubscriptionsReader {
    pub fn new(
        guid_prefix: GuidPrefix,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    ) -> StatefulReader {
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let expects_inline_qos = true;
        let heartbeat_response_delay = Duration::from_millis(500);
        let heartbeat_supression_duration = DURATION_ZERO;

        StatefulReader::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        )
    }
}

pub struct SEDPBuiltinTopicsWriter;

impl SEDPBuiltinTopicsWriter {
    pub fn new(
        guid_prefix: GuidPrefix,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        heartbeat_period: Duration,
    ) -> StatefulWriter {
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let push_mode = true;
        let nack_response_delay = Duration::from_millis(200);
        let nack_suppression_duration = DURATION_ZERO;
        let data_max_sized_serialized = None;

        StatefulWriter::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_sized_serialized,
        )
    }
}
pub struct SEDPBuiltinTopicsReader;

impl SEDPBuiltinTopicsReader {
    pub fn new(
        guid_prefix: GuidPrefix,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    ) -> StatefulReader {
        let guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let expects_inline_qos = true;
        let heartbeat_response_delay = Duration::from_millis(500);
        let heartbeat_supression_duration = DURATION_ZERO;

        StatefulReader::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        )
    }
}
