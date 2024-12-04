use super::{
    history_cache::HistoryCache,
    reader::{TransportStatefulReader, TransportStatelessReader},
    types::{EntityId, Guid, Locator, ProtocolVersion, ReliabilityKind, TopicKind, VendorId},
    writer::{TransportStatefulWriter, TransportStatelessWriter},
};

pub trait TransportParticipant: Send + Sync {
    fn guid(&self) -> Guid;
    fn protocol_version(&self) -> ProtocolVersion;
    fn vendor_id(&self) -> VendorId;
    fn metatraffic_unicast_locator_list(&self) -> &[Locator];
    fn metatraffic_multicast_locator_list(&self) -> &[Locator];
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];

    fn create_stateless_reader(
        &mut self,
        entity_id: EntityId,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportStatelessReader>;

    fn create_stateless_writer(
        &mut self,
        entity_id: EntityId,
        topic_kind: TopicKind,
    ) -> Box<dyn TransportStatelessWriter>;

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        topic_kind: TopicKind,
        reliability_kind: ReliabilityKind,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportStatefulReader>;

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        topic_kind: TopicKind,
        reliability_kind: ReliabilityKind,
    ) -> Box<dyn TransportStatefulWriter>;
}
