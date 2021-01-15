use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    rtps::{
        behavior::{ReaderProxy, WriterProxy},
        endpoint_types::BuiltInEndpointSet,
        messages::types::Count,
        types::{GuidPrefix, Locator, ProtocolVersion, VendorId},
    },
    types::{DDSType, Data, DomainId, Duration, InstanceHandle, TopicKind},
};

pub struct ParticipantProxy {
    pub domain_id: DomainId,
    pub domain_tag: String,
    pub protocol_version: ProtocolVersion,
    pub guid_prefix: GuidPrefix,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub available_built_in_endpoints: BuiltInEndpointSet,
    // built_in_endpoint_qos:
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub manual_liveliness_count: Count,
}

pub struct SpdpDiscoveredParticipantData {
    pub dds_participant_data: ParticipantBuiltinTopicData,
    pub participant_proxy: ParticipantProxy,
    pub lease_duration: Duration,
}
impl DDSType for SpdpDiscoveredParticipantData {
    fn type_name() -> &'static str {
        "SpdpDiscoveredParticipantData"
    }

    fn topic_kind() -> TopicKind {
        TopicKind::WithKey
    }

    fn instance_handle(&self) -> InstanceHandle {
        [
            b't', b'o', b'd', b'o', b':', b'i', b'n', b's', b't', b'a', b'n', b'c', b'e', b'h',
            b'a', b'n',
        ]
    }

    fn serialize(&self) -> Data {
        vec![
            b't', b'o', b'd', b'o', b':', b'i', b'm', b'p', b'l', b'e', b'm', b'e', b'n', b't',
            b' ', b's', b'e', b'r', b'i', b'a', b'l', b'i', b'z', b'e',
        ]
    }

    fn deserialize(_data: Data) -> Self {
        todo!()
    }
}

pub struct DiscoveredTopicData {
    pub dds_topic_data: TopicBuiltinTopicData,
}

pub struct DiscoveredWriterData {
    pub dds_writer_data: PublicationBuiltinTopicData,
    pub writer_proxy: WriterProxy,
}

pub struct DiscoveredReaderData {
    pub dds_reader_data: SubscriptionBuiltinTopicData,
    pub reader_proxy: ReaderProxy,
}
