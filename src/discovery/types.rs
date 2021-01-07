use crate::{builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData}, types::{self, Data}};
use crate::rtps::behavior::types::Duration;
use crate::rtps::behavior::{ReaderProxy, WriterProxy};
use crate::rtps::messages::types::Count;
use crate::rtps::types::{ProtocolVersion, GuidPrefix, Locator, VendorId};
use crate::rtps::endpoint_types::BuiltInEndpointSet;
use crate::types::{DDSType, DomainId};

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
        [5; 16]
    }

    fn serialize(&self) -> types::Data {
        vec![0,0,0,1, 1,2,3,4, 5, 6, 7, 8]
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