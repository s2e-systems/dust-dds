use super::parameter_id_values::{
    PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_GROUP_ENTITYID, PID_MULTICAST_LOCATOR,
    PID_UNICAST_LOCATOR,
};
use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    infrastructure::type_support::TypeSupport,
    transport::types::{EntityId, Guid, Locator},
};
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "Mutable")]
pub struct ReaderProxy {
    #[dust_dds(id=PID_ENDPOINT_GUID)]
    pub remote_reader_guid: Guid,
    #[dust_dds(id=PID_GROUP_ENTITYID)]
    pub remote_group_entity_id: EntityId,
    #[dust_dds(id=PID_UNICAST_LOCATOR)]
    pub unicast_locator_list: Vec<Locator>,
    #[dust_dds(id=PID_MULTICAST_LOCATOR)]
    pub multicast_locator_list: Vec<Locator>,
    #[dust_dds(id=PID_EXPECTS_INLINE_QOS)]
    pub expects_inline_qos: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
pub struct DiscoveredReaderData {
    pub(crate) dds_subscription_data: SubscriptionBuiltinTopicData,
    pub(crate) reader_proxy: ReaderProxy,
}
