use super::{
    history_cache::HistoryCache,
    reader::{TransportStatefulReader, TransportStatelessReader},
    types::{EntityId, Guid, Locator, ProtocolVersion, ReliabilityKind, VendorId},
    writer::{TransportStatefulWriter, TransportStatelessWriter},
};

pub trait Transport: Send + Sync {
    fn create_participant(&self) -> Box<dyn TransportParticipant>;
}

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
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportStatelessReader>;

    fn create_stateless_writer(
        &mut self,
        entity_id: EntityId,
        data_max_size_serialized: usize,
    ) -> Box<dyn TransportStatelessWriter>;

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportStatefulReader>;

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
        data_max_size_serialized: usize,
    ) -> Box<dyn TransportStatefulWriter>;
}
