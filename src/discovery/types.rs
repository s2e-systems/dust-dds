use crate::{builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData}, types};
use crate::rtps::behavior::types::Duration;
use crate::rtps::behavior::{ReaderProxy, WriterProxy};
use crate::rtps::messages::types::Count;
use crate::rtps::types::{ProtocolVersion, GuidPrefix, Locator, VendorId};
use crate::rtps::endpoint_types::BuiltInEndpointSet;
use crate::types::{DDSType, DomainId};

pub struct ParticipantProxy {
    domain_id: DomainId,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    guid_prefix: GuidPrefix,
    vendor_id: VendorId,
    expects_inline_qos: bool,
    available_built_in_endpoints: BuiltInEndpointSet,
    // built_in_endpoint_qos: 
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    manual_liveliness_count: Count,
}

pub struct SpdpDiscoveredParticipantData{
    pub dds_participant_data: ParticipantBuiltinTopicData,
    pub participant_proxy: ParticipantProxy,
    pub lease_duration: Duration,
}
impl DDSType for SpdpDiscoveredParticipantData {
    fn type_name() -> &'static str {
        "SpdpDiscoveredParticipantData"
    }

    fn topic_kind() -> types::TopicKind {
        types::TopicKind::WithKey
    }

    fn instance_handle(&self) -> types::InstanceHandle {
        todo!()
    }

    fn serialize(&self) -> types::Data {
        todo!()
    }

    fn deserialize(data: types::Data) -> Self {
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