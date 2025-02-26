use super::types::{DurabilityKind, EntityId, Guid, Locator, ReliabilityKind};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WriterProxy {
    pub remote_writer_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub reliability_kind: ReliabilityKind,
    pub durability_kind: DurabilityKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
}

pub trait TransportStatelessReader: Send + Sync {
    fn guid(&self) -> Guid;
}

pub trait TransportStatefulReader: Send + Sync {
    fn guid(&self) -> Guid;
    fn is_historical_data_received(&self) -> bool;
    fn add_matched_writer(&mut self, writer_proxy: WriterProxy);
    fn remove_matched_writer(&mut self, remote_writer_guid: Guid);
}
