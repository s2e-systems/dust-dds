use super::types::{
    CacheChange, EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, ReaderProxy,
    ReliabilityKind, VendorId, WriterProxy,
};
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub trait TransportParticipantFactory: Send + 'static {
    type TransportParticipant: TransportParticipant;

    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> impl Future<Output = Self::TransportParticipant> + Send;
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
    fn is_change_acknowledged(&self, sequence_number: i64) -> impl Future<Output = bool> + Send;
    fn add_matched_reader(&mut self, reader_proxy: ReaderProxy) -> impl Future<Output = ()> + Send;
    fn remove_matched_reader(
        &mut self,
        remote_reader_guid: Guid,
    ) -> impl Future<Output = ()> + Send;
}

pub trait TransportStatelessReader: Send + Sync {
    fn guid(&self) -> Guid;
}

pub trait TransportStatefulReader: Send + Sync {
    fn guid(&self) -> Guid;
    fn is_historical_data_received(&self) -> impl Future<Output = bool> + Send;
    fn add_matched_writer(&mut self, writer_proxy: WriterProxy) -> impl Future<Output = ()> + Send;
    fn remove_matched_writer(
        &mut self,
        remote_writer_guid: Guid,
    ) -> impl Future<Output = ()> + Send;
}

pub trait HistoryCache: Send {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    fn remove_change(&mut self, sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

pub trait TransportParticipant: Send {
    type StatelessReader: TransportStatelessReader;
    type StatefulReader: TransportStatefulReader;
    type StatelessWriter: TransportStatelessWriter;
    type StatefulWriter: TransportStatefulWriter;

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
    ) -> Self::StatelessReader;

    fn create_stateless_writer(&mut self, entity_id: EntityId) -> Self::StatelessWriter;

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Self::StatefulReader;

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
    ) -> Self::StatefulWriter;
}
