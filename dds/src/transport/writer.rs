use super::{
    history_cache::HistoryCache,
    types::{DurabilityKind, EntityId, Guid, Locator, ReliabilityKind},
};
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReaderProxy {
    pub remote_reader_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub reliability_kind: ReliabilityKind,
    pub durability_kind: DurabilityKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub expects_inline_qos: bool,
}

pub trait TransportStatelessWriter: Send + Sync {
    fn guid(&self) -> Guid;

    fn history_cache(&mut self) -> &mut dyn HistoryCache;

    fn add_reader_locator(&mut self, locator: Locator);
    fn remove_reader_locator(&mut self, locator: &Locator);
}

pub trait TransportStatefulWriter: Send + Sync {
    fn guid(&self) -> Guid;

    fn history_cache(&mut self) -> &mut dyn HistoryCache;

    fn is_change_acknowledged(&self, sequence_number: i64) -> bool;

    fn add_matched_reader(&mut self, reader_proxy: ReaderProxy);

    fn remove_matched_reader(&mut self, remote_reader_guid: Guid);
}
