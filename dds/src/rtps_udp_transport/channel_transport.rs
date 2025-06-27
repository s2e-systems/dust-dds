use crate::{
    rtps::message_sender::Clock,
    rtps_messages::overall_structure::RtpsMessageWrite,
    std_runtime::executor::block_on,
    transport::{
        interface::{
            HistoryCache, TransportParticipant, TransportParticipantFactory,
            TransportStatefulReader, TransportStatefulWriter, TransportStatelessReader,
            TransportStatelessWriter,
        },
        types::{CacheChange, ReaderProxy, WriterProxy},
    },
};
use core::{
    future::{Future, IntoFuture},
    pin::Pin,
};
use dust_dds::{
    rtps::{
        message_sender::WriteMessage,
        stateful_reader::RtpsStatefulReader,
        stateful_writer::RtpsStatefulWriter,
        stateless_reader::RtpsStatelessReader,
        stateless_writer::RtpsStatelessWriter,
        types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    },
    transport::types::{
        EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, ReliabilityKind, VendorId,
        ENTITYID_PARTICIPANT,
    },
};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

pub struct RtpsChannelTransportParticipantFactory {}

impl Default for RtpsChannelTransportParticipantFactory {
    fn default() -> Self {
        Self {}
    }
}

impl TransportParticipantFactory for RtpsChannelTransportParticipantFactory {
    type TransportParticipant = RtpsChannelTransportParticipant;

    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> Self::TransportParticipant {
        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);
        let participant = RtpsChannelTransportParticipant {
            guid,
            stateless_reader: None,
            stateless_writer: None,
        };

        participant
    }
}

pub struct RtpsChannelTransportClock;

impl Clock for RtpsChannelTransportClock {
    fn now(&self) -> core::time::Duration {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Clock should always give valid Unix time")
    }
}

impl TransportStatelessReader for RtpsStatelessReader {
    fn guid(&self) -> Guid {
        self.guid()
    }
}

pub struct RtpsChannelTransportParticipant {
    guid: Guid,
    stateless_reader: Option<RtpsStatelessReader>,
    stateless_writer: Option<RtpsChannelTransportStatelessWriter>,
}

pub struct RtpsChannelTransportStatelessWriter {
    rtps_stateless_writer: RtpsStatelessWriter,
    rtps_stateless_reader: RtpsStatelessReader,
}
impl HistoryCache for RtpsChannelTransportStatelessWriter {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self.rtps_stateless_writer.add_change(cache_change);
        crate::rtps::stateless_writer::behavior(
            &mut self.rtps_stateless_writer,
            &self.rtps_stateless_reader,
        );
        Box::pin(async {})
    }

    fn remove_change(&mut self, sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}
impl TransportStatelessWriter for RtpsChannelTransportStatelessWriter {
    fn guid(&self) -> Guid {
        self.rtps_stateless_writer.guid()
    }

    fn history_cache(&mut self) -> &mut dyn HistoryCache {
        self
    }

    fn add_reader_locator(&mut self, locator: Locator) {
        todo!()
    }

    fn remove_reader_locator(&mut self, locator: &Locator) {
        todo!()
    }
}
impl WriteMessage for RtpsStatelessReader {
    fn write_message(&self, datagram: &[u8], _locator_list: &[Locator]) {
        // block_on(async { self.process_message(datagram).await.unwrap() });

    }

    fn guid_prefix(&self) -> GuidPrefix {
        // self.rtps_stateless_writer.guid().prefix()
        todo!()
    }
}

impl TransportStatelessReader for Guid {
    fn guid(&self) -> Guid {
        *self
    }
}
impl TransportParticipant for RtpsChannelTransportParticipant {
    type HistoryCache = Box<dyn HistoryCache + Sync>;
    type StatelessReader = Guid;
    type StatelessWriter = RtpsChannelTransportStatelessWriter;
    type StatefulReader = Box<dyn TransportStatefulReader>;
    type StatefulWriter = Box<dyn TransportStatefulWriter>;

    fn guid(&self) -> Guid {
        self.guid
    }
    fn protocol_version(&self) -> ProtocolVersion {
        PROTOCOLVERSION
    }
    fn vendor_id(&self) -> VendorId {
        VENDOR_ID_S2E
    }
    fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        &[]
    }
    fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &[]
    }
    fn default_unicast_locator_list(&self) -> &[Locator] {
        &[]
    }
    fn default_multicast_locator_list(&self) -> &[Locator] {
        &[]
    }
    fn create_stateless_reader(
        &mut self,
        entity_id: EntityId,
        reader_history_cache: Self::HistoryCache,
    ) -> Self::StatelessReader {
        let guid = Guid::new(self.guid.prefix(), entity_id);
        self.stateless_reader
            .replace(RtpsStatelessReader::new(guid, reader_history_cache));
        guid
    }
    fn create_stateless_writer(&mut self, entity_id: EntityId) -> Self::StatelessWriter {
        let guid = Guid::new(self.guid.prefix(), entity_id);
        let rtps_stateless_writer = RtpsStatelessWriter::new(guid);
        let rtps_stateless_reader = self
            .stateless_reader
            .take()
            .expect("statelessreader already created");
        RtpsChannelTransportStatelessWriter {
            rtps_stateless_writer,
            rtps_stateless_reader,
        }
    }

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
        reader_history_cache: Self::HistoryCache,
    ) -> Self::StatefulReader {
        todo!()
    }

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        _reliability_kind: ReliabilityKind,
    ) -> Self::StatefulWriter {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::types::ENTITYID_UNKNOWN;
    use dust_dds::transport::types::ChangeKind;
    use std::sync::mpsc::{sync_channel, SyncSender};

    #[test]
    fn basic_transport_stateless_reader_writer_usage() {
        let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let domain_id = 0;
        let transport = RtpsChannelTransportParticipantFactory::default();
        let mut participant = transport.create_participant(guid_prefix, domain_id);

        struct MockHistoryCache(SyncSender<CacheChange>);

        impl HistoryCache for MockHistoryCache {
            fn add_change(
                &mut self,
                cache_change: CacheChange,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                self.0.send(cache_change).unwrap();
                Box::pin(async {})
            }
            fn remove_change(
                &mut self,
                _sequence_number: i64,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                Box::pin(async {
                    unimplemented!();
                })
            }
        }

        let entity_id = EntityId::new([1, 2, 3], 4);
        let (sender, receiver) = sync_channel(0);
        let reader_history_cache = Box::new(MockHistoryCache(sender));
        let _reader = participant.create_stateless_reader(entity_id, reader_history_cache);

        let entity_id = EntityId::new([5, 6, 7], 8);
        let mut writer = participant.create_stateless_writer(entity_id);

        for locator in participant.default_unicast_locator_list() {
            writer.add_reader_locator(locator.clone());
        }

        let cache_change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: writer.guid(),
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: None,
            data_value: vec![0, 0, 0, 0, 1, 2, 3, 4].into(),
        };
        block_on(async {
            writer
                .history_cache()
                .add_change(cache_change.clone())
                .await
        });

        let received_cache_change = receiver
            .recv_timeout(std::time::Duration::from_secs(30))
            .unwrap();
        assert_eq!(cache_change, received_cache_change);
    }

    // #[test]
    // fn basic_transport_stateful_reader_writer_usage() {
    //     let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    //     let domain_id = 0;
    //     let transport = RtpsChannelTransportParticipantFactory::default();
    //     let mut participant = transport.create_participant(guid_prefix, domain_id);

    //     struct MockHistoryCache(SyncSender<CacheChange>);

    //     impl HistoryCache for MockHistoryCache {
    //         fn add_change(
    //             &mut self,
    //             cache_change: CacheChange,
    //         ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    //             self.0.send(cache_change).unwrap();
    //             Box::pin(async {})
    //         }
    //         fn remove_change(
    //             &mut self,
    //             _sequence_number: i64,
    //         ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    //             Box::pin(async {
    //                 unimplemented!();
    //             })
    //         }
    //     }

    //     let entity_id = EntityId::new([1, 2, 3], 4);
    //     let reliability_kind = ReliabilityKind::BestEffort;
    //     let (sender, receiver) = sync_channel(0);
    //     let reader_history_cache = Box::new(MockHistoryCache(sender));
    //     let mut reader =
    //         participant.create_stateful_reader(entity_id, reliability_kind, reader_history_cache);

    //     let entity_id = EntityId::new([5, 6, 7], 8);
    //     let mut writer = participant.create_stateful_writer(entity_id, reliability_kind);

    //     let reader_proxy = ReaderProxy {
    //         remote_reader_guid: reader.guid(),
    //         remote_group_entity_id: ENTITYID_UNKNOWN,
    //         reliability_kind,
    //         durability_kind: DurabilityKind::Volatile,
    //         unicast_locator_list: vec![],
    //         multicast_locator_list: vec![],
    //         expects_inline_qos: false,
    //     };
    //     writer.add_matched_reader(reader_proxy);

    //     let writer_proxy = WriterProxy {
    //         remote_writer_guid: writer.guid(),
    //         remote_group_entity_id: ENTITYID_UNKNOWN,
    //         reliability_kind,
    //         durability_kind: DurabilityKind::Volatile,
    //         unicast_locator_list: vec![],
    //         multicast_locator_list: vec![],
    //     };
    //     reader.add_matched_writer(writer_proxy);
    //     let cache_change = CacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: writer.guid(),
    //         sequence_number: 1,
    //         source_timestamp: None,
    //         instance_handle: None,
    //         data_value: vec![0, 0, 0, 0, 1, 2, 3, 4].into(),
    //     };

    //     block_on(async {
    //         writer
    //             .history_cache()
    //             .add_change(cache_change.clone())
    //             .await
    //     });

    //     let received_cache_change = receiver
    //         .recv_timeout(std::time::Duration::from_secs(3))
    //         .unwrap();
    //     assert_eq!(cache_change, received_cache_change);
    // }
}
