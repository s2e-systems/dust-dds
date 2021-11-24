use crate::{
    behavior::{
        reader::reader::RtpsReader,
        types::{Duration, DURATION_ZERO},
        writer::stateful_writer::RtpsStatefulWriter,
    },
    structure::{
        history_cache::RtpsHistoryCacheConstructor,
        types::{
            EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, BUILT_IN_READER_WITH_KEY,
            BUILT_IN_WRITER_WITH_KEY,
        },
    },
};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_READER_WITH_KEY);

const DEFAULT_HEARTBEAT_PERIOD: Duration = Duration {
    seconds: 2,
    fraction: 0,
};

const DEFAULT_NACK_RESPONSE_DELAY: Duration = Duration {
    seconds: 0,
    fraction: 200,
};

const DEFAULT_NACK_SUPPRESSION_DURATION: Duration = DURATION_ZERO;

const DEFAULT_HEARTBEAT_RESPONSE_DELAY: Duration = Duration {
    seconds: 0,
    fraction: 500,
};

const DEFAULT_HEARTBEAT_SUPRESSION_DURATION: Duration = DURATION_ZERO;

pub struct SedpBuiltinPublicationsWriter;

impl SedpBuiltinPublicationsWriter {
    pub fn create<L, C, R>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsStatefulWriter<L, C, R>
    where
        C: RtpsHistoryCacheConstructor,
        R: Default,
    {
        let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let push_mode = true;
        let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
        let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
        let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
        let data_max_size_serialized = None;
        RtpsStatefulWriter::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        )
    }
}

pub struct SedpBuiltinPublicationsReader;

impl SedpBuiltinPublicationsReader {
    pub fn create<L, C>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsReader<L, C>
    where
        C: RtpsHistoryCacheConstructor,
    {
        let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
        let heartbeat_supression_duration = DEFAULT_HEARTBEAT_SUPRESSION_DURATION;
        let expects_inline_qos = false;
        RtpsReader::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        )
    }
}

pub struct SedpBuiltinSubscriptionsWriter;

impl SedpBuiltinSubscriptionsWriter {
    pub fn create<L, C, R>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsStatefulWriter<L, C, R>
    where
        C: RtpsHistoryCacheConstructor,
        R: Default,
    {
        let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let push_mode = true;
        let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
        let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
        let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
        let data_max_size_serialized = None;
        RtpsStatefulWriter::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        )
    }
}

pub struct SedpBuiltinSubscriptionsReader;

impl SedpBuiltinSubscriptionsReader {
    pub fn create<L, C>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsReader<L, C>
    where
        C: RtpsHistoryCacheConstructor,
    {
        let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
        let heartbeat_supression_duration = DEFAULT_HEARTBEAT_SUPRESSION_DURATION;
        let expects_inline_qos = false;
        RtpsReader::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        )
    }
}

pub struct SedpBuiltinTopicsWriter;

impl SedpBuiltinTopicsWriter {
    pub fn create<L, C, R>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsStatefulWriter<L, C, R>
    where
        C: RtpsHistoryCacheConstructor,
        R: Default,
    {
        let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let push_mode = true;
        let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD;
        let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY;
        let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION;
        let data_max_size_serialized = None;
        RtpsStatefulWriter::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        )
    }
}

pub struct SedpBuiltinTopicsReader;

impl SedpBuiltinTopicsReader {
    pub fn create<L, C>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsReader<L, C>
    where
        C: RtpsHistoryCacheConstructor,
    {
        let guid = Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::Reliable;
        let heartbeat_response_delay = DEFAULT_HEARTBEAT_RESPONSE_DELAY;
        let heartbeat_supression_duration = DEFAULT_HEARTBEAT_SUPRESSION_DURATION;
        let expects_inline_qos = false;
        RtpsReader::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        )
    }
}
