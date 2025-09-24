use super::parameter_id_values::{
    PID_ENDPOINT_GUID, PID_GROUP_ENTITYID, PID_MULTICAST_LOCATOR, PID_UNICAST_LOCATOR,
};
use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    infrastructure::type_support::TypeSupport,
    transport::types::{EntityId, Guid, Locator},
};
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "Mutable")]
pub struct WriterProxy {
    #[dust_dds(id=PID_ENDPOINT_GUID)]
    pub remote_writer_guid: Guid,
    #[dust_dds(id=PID_GROUP_ENTITYID)]
    pub remote_group_entity_id: EntityId,
    #[dust_dds(id=PID_UNICAST_LOCATOR)]
    pub unicast_locator_list: Vec<Locator>,
    #[dust_dds(id=PID_MULTICAST_LOCATOR)]
    pub multicast_locator_list: Vec<Locator>,
}

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
pub struct DiscoveredWriterData {
    pub(crate) dds_publication_data: PublicationBuiltinTopicData,
    pub(crate) writer_proxy: WriterProxy,
}
