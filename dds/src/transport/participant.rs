use super::types::{EntityId, Guid, Locator, ProtocolVersion, ReliabilityKind, VendorId};

pub trait TransportParticipant: Send {
    type HistoryCache;
    type StatelessReader;
    type StatefulReader;
    type StatelessWriter;
    type StatefulWriter;

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
        reader_history_cache: Self::HistoryCache,
    ) -> Self::StatelessReader;

    fn create_stateless_writer(&mut self, entity_id: EntityId) -> Self::StatelessWriter;

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
        reader_history_cache: Self::HistoryCache,
    ) -> Self::StatefulReader;

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
    ) -> Self::StatefulWriter;
}
