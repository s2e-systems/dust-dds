use super::types::{
    CacheChange, EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, ReliabilityKind, VendorId,
};
use crate::rtps_udp_transport::udp_transport::{
    RtpsTransportStatefulReader, RtpsTransportStatefulWriter, RtpsTransportStatelessWriter,
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

pub trait HistoryCache: Send {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    fn remove_change(&mut self, sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

pub trait TransportParticipant: Send {
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
    ) -> impl Future<Output = Guid> + Send;

    fn create_stateless_writer(
        &mut self,
        entity_id: EntityId,
    ) -> impl Future<Output = RtpsTransportStatelessWriter> + Send;

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> impl Future<Output = RtpsTransportStatefulReader> + Send;

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
    ) -> impl Future<Output = RtpsTransportStatefulWriter> + Send;
}
