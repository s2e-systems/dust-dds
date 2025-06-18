use crate::transport::types::CacheChange;

use super::types::{
    ChangeKind, DurabilityKind, EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, ReaderProxy,
    ReliabilityKind, Time, VendorId, WriterProxy,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{future::Future, pin::Pin};

pub trait TransportParticipantFactory: Send {
    type TransportParticipant;

    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> Self::TransportParticipant;
}
pub trait TransportStatelessWriter: Send {
    fn guid(&self) -> Guid;
    fn history_cache(&mut self) -> &mut dyn HistoryCache;
    fn add_reader_locator(&mut self, locator: Locator);
    fn remove_reader_locator(&mut self, locator: &Locator);
}

pub trait TransportStatefulWriter: Send {
    fn guid(&self) -> Guid;
    fn history_cache(&mut self) -> &mut dyn HistoryCache;
    fn is_change_acknowledged(&self, sequence_number: i64) -> bool;
    fn add_matched_reader(&mut self, reader_proxy: ReaderProxy);
    fn remove_matched_reader(&mut self, remote_reader_guid: Guid);
}

pub trait TransportStatelessReader: Send {
    fn guid(&self) -> Guid;
}

pub trait TransportStatefulReader: Send {
    fn guid(&self) -> Guid;
    fn is_historical_data_received(&self) -> bool;
    fn add_matched_writer(&mut self, writer_proxy: WriterProxy);
    fn remove_matched_writer(&mut self, remote_writer_guid: Guid);
}

impl CacheChange {
    pub fn kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn sequence_number(&self) -> i64 {
        self.sequence_number
    }

    pub fn source_timestamp(&self) -> Option<Time> {
        self.source_timestamp
    }

    pub fn data_value(&self) -> &Arc<[u8]> {
        &self.data_value
    }
}

pub trait HistoryCache: Send {
    fn add_change(&mut self, cache_change: CacheChange)
        -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn remove_change(&mut self, sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

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
