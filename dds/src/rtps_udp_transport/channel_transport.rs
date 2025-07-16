use crate::{
    rtps::{
        message_sender::{Clock, WriteMessage},
        stateful_reader::RtpsStatefulReader,
        stateful_writer::{stateful_writer_write_message, RtpsStatefulWriter},
        stateless_reader::RtpsStatelessReader,
        stateless_writer::RtpsStatelessWriter,
        types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    },
    std_runtime::executor::block_on,
    transport::{
        interface::{
            HistoryCache, TransportParticipant, TransportParticipantFactory,
            TransportStatefulReader, TransportStatefulWriter, TransportStatelessReader,
            TransportStatelessWriter,
        },
        types::{
            CacheChange, EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, ReliabilityKind,
            VendorId, ENTITYID_PARTICIPANT,
        },
    },
};
use async_lock::Mutex;
use std::{future::Future, pin::Pin, sync::Arc};

#[derive(Default)]
pub struct RtpsChannelTransportParticipantFactory {}

impl TransportParticipantFactory for RtpsChannelTransportParticipantFactory {
    type TransportParticipant = RtpsChannelTransportParticipant;

    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        _domain_id: i32,
    ) -> Self::TransportParticipant {
        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);
        RtpsChannelTransportParticipant {
            guid,
            stateless_reader: None,
            stateful_reader: None,
        }
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

pub struct RtpsChannelTransportStatelessWriter {
    rtps_stateless_writer: RtpsStatelessWriter,
    rtps_stateless_reader: Mutex<RtpsStatelessReader>,
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

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
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
        self.rtps_stateless_writer.reader_locator_add(locator)
    }

    fn remove_reader_locator(&mut self, _locator: &Locator) {
        todo!()
    }
}
impl WriteMessage for Mutex<RtpsStatelessReader> {
    fn write_message(&self, datagram: &[u8], _locator_list: &[Locator]) {
        block_on(async { self.lock().await.process_message(datagram).await.unwrap() });
    }

    fn guid_prefix(&self) -> GuidPrefix {
        block_on(async { self.lock().await.guid().prefix() })
    }
}

impl WriteMessage for Arc<Mutex<RtpsStatefulReader>> {
    fn write_message(&self, datagram: &[u8], _locator_list: &[Locator]) {
        block_on(async {
            self.lock().await.process_message(datagram, self).await.ok();
        });
    }

    fn guid_prefix(&self) -> GuidPrefix {
        block_on(async { self.lock().await.guid().prefix() })
    }
}

impl TransportStatefulReader for Arc<Mutex<RtpsStatefulReader>> {
    fn guid(&self) -> Guid {
        block_on(async { self.lock().await.guid() })
    }

    fn is_historical_data_received(&self) -> bool {
        todo!()
    }

    fn add_matched_writer(&mut self, writer_proxy: crate::transport::types::WriterProxy) {
        block_on(async { self.lock().await.add_matched_writer(&writer_proxy) })
    }

    fn remove_matched_writer(&mut self, _remote_writer_guid: Guid) {
        todo!()
    }
}

pub struct RtpsChannelTransportStatefulWriter {
    rtps_stateful_writer: RtpsStatefulWriter,
    rtps_stateful_reader: Arc<Mutex<RtpsStatefulReader>>,
}
impl HistoryCache for RtpsChannelTransportStatefulWriter {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self.rtps_stateful_writer.add_change(cache_change);
        block_on(async {
            stateful_writer_write_message(
                &mut self.rtps_stateful_writer,
                &self.rtps_stateful_reader,
                &RtpsChannelTransportClock,
            )
            .await
        });
        Box::pin(async {})
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}
impl TransportStatefulWriter for RtpsChannelTransportStatefulWriter {
    fn guid(&self) -> Guid {
        self.rtps_stateful_writer.guid()
    }

    fn history_cache(&mut self) -> &mut dyn HistoryCache {
        self
    }

    fn is_change_acknowledged(&self, sequence_number: i64) -> bool {
        self.rtps_stateful_writer
            .is_change_acknowledged(sequence_number)
    }

    fn add_matched_reader(&mut self, reader_proxy: crate::transport::types::ReaderProxy) {
        self.rtps_stateful_writer.add_matched_reader(&reader_proxy);
    }

    fn remove_matched_reader(&mut self, _remote_reader_guid: Guid) {}
}

impl TransportStatelessReader for Guid {
    fn guid(&self) -> Guid {
        *self
    }
}

pub struct RtpsChannelTransportParticipant {
    guid: Guid,
    stateless_reader: Option<RtpsStatelessReader>,
    stateful_reader: Option<Arc<Mutex<RtpsStatefulReader>>>,
}

impl TransportParticipant for RtpsChannelTransportParticipant {
    type HistoryCache = Box<dyn HistoryCache + Sync>;
    type StatelessReader = Guid;
    type StatelessWriter = RtpsChannelTransportStatelessWriter;
    type StatefulReader = Arc<Mutex<RtpsStatefulReader>>;
    type StatefulWriter = RtpsChannelTransportStatefulWriter;

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
            .expect("statelessreader must be already created");
        RtpsChannelTransportStatelessWriter {
            rtps_stateless_writer,
            rtps_stateless_reader: Mutex::new(rtps_stateless_reader),
        }
    }

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
        reader_history_cache: Self::HistoryCache,
    ) -> Self::StatefulReader {
        let guid = Guid::new(self.guid.prefix(), entity_id);
        let stateful_reader = Arc::new(Mutex::new(RtpsStatefulReader::new(
            guid,
            reader_history_cache,
            reliability_kind,
        )));
        self.stateful_reader.replace(stateful_reader.clone());
        stateful_reader
    }

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        _reliability_kind: ReliabilityKind,
    ) -> Self::StatefulWriter {
        let guid = Guid::new(self.guid.prefix(), entity_id);
        let data_max_size_serialized = 1024;
        let rtps_stateful_writer = RtpsStatefulWriter::new(guid, data_max_size_serialized);
        let rtps_stateful_reader = self
            .stateful_reader
            .take()
            .expect("statefulreader must be already created");
        RtpsChannelTransportStatefulWriter {
            rtps_stateful_writer,
            rtps_stateful_reader,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        std_runtime::executor::block_on,
        transport::types::{DurabilityKind, ReaderProxy, WriterProxy, ENTITYID_UNKNOWN},
    };
    use dust_dds::transport::types::ChangeKind;
    use std::sync::mpsc::{channel, Sender};
    #[ignore]
    #[test]
    fn basic_transport_stateless_reader_writer_usage() {
        let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let domain_id = 0;
        let transport = RtpsChannelTransportParticipantFactory::default();
        let mut participant = transport.create_participant(guid_prefix, domain_id);

        struct MockHistoryCache(Sender<CacheChange>);

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
        let (sender, receiver) = channel();
        let reader_history_cache = Box::new(MockHistoryCache(sender));
        let _reader = participant.create_stateless_reader(entity_id, reader_history_cache);

        let entity_id = EntityId::new([5, 6, 7], 8);
        let mut writer = participant.create_stateless_writer(entity_id);

        writer.add_reader_locator(Locator::new(0, 0, [0; 16]));

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

    #[test]
    fn basic_transport_stateful_reader_writer_usage() {
        let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let domain_id = 0;
        let transport = RtpsChannelTransportParticipantFactory::default();
        let mut participant = transport.create_participant(guid_prefix, domain_id);

        struct MockHistoryCache(Sender<CacheChange>);

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
        let reliability_kind = ReliabilityKind::BestEffort;
        let (sender, receiver) = channel();
        let reader_history_cache = Box::new(MockHistoryCache(sender));
        let mut reader =
            participant.create_stateful_reader(entity_id, reliability_kind, reader_history_cache);

        let entity_id = EntityId::new([5, 6, 7], 8);
        let mut writer = participant.create_stateful_writer(entity_id, reliability_kind);

        let reader_proxy = ReaderProxy {
            remote_reader_guid: reader.lock().unwrap().guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        };
        writer.add_matched_reader(reader_proxy);

        let writer_proxy = WriterProxy {
            remote_writer_guid: writer.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
        };
        reader.add_matched_writer(writer_proxy);

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
            .recv_timeout(std::time::Duration::from_secs(3))
            .unwrap();
        assert_eq!(cache_change, received_cache_change);
    }
}
