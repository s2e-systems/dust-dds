use crate::{
    rtps::message_sender::Clock,
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
use async_lock::Mutex;
use core::{future::Future, pin::Pin};
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
        ENTITYID_PARTICIPANT, LOCATOR_KIND_UDP_V4,
    },
};
use std::sync::{
    mpsc::{channel, Sender},
    Arc,
};

pub struct RtpsChannelTransportParticipantFactory;

impl Default for RtpsChannelTransportParticipantFactory {
    fn default() -> Self {
        Self {}
    }
}

enum ChannelMessageKind {
    AddStatelessReader(RtpsStatelessReader),
    AddStatefulReader(Arc<Mutex<RtpsStatefulReader>>),
    AddStatefulWriter(Arc<Mutex<RtpsStatefulWriter>>),
    MetatrafficMulticastSocket(Arc<[u8]>),
    MetatrafficUnicastSocket(Arc<[u8]>),
    DefaultUnicastSocket(Arc<[u8]>),
    Poke,
}

// static CHANNEL_LIST: [channel()] = ;

impl TransportParticipantFactory for RtpsChannelTransportParticipantFactory {
    type TransportParticipant = Box<
        dyn TransportParticipant<
            HistoryCache = Box<dyn HistoryCache>,
            StatelessReader = Box<dyn TransportStatelessReader>,
            StatefulReader = Box<dyn TransportStatefulReader>,
            StatelessWriter = Box<dyn TransportStatelessWriter>,
            StatefulWriter = Box<dyn TransportStatefulWriter>,
        >,
    >;

    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> Self::TransportParticipant {
        // let metattrafic_unicast_locator_port = metatraffic_unicast_socket
        //     .local_addr()
        //     .unwrap()
        //     .port()
        //     .into();
        // let metatraffic_unicast_locator_list: Vec<Locator> = interface_address_list
        //     .clone()
        //     .map(|a| Locator::from_ip_and_port(&a, metattrafic_unicast_locator_port))
        //     .collect();

        // // Open socket for multicast metatraffic data
        // let metatraffic_multicast_locator_list = vec![Locator::new(
        //     LOCATOR_KIND_UDP_V4,
        //     port_builtin_multicast(domain_id) as u32,
        //     DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        // )];

        // let metatraffic_multicast_socket = Arc::new(
        //     get_multicast_socket(
        //         DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        //         port_builtin_multicast(domain_id),
        //         interface_address_list,
        //     )
        //     .unwrap(),
        // );

        let message_writer = Arc::new(MessageWriter::new(guid_prefix));

        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);

        // let (chanel_message_sender, chanel_message_receiver) = channel();

        let participant = RtpsChannelTransportParticipant {
            guid,
            message_writer: message_writer.clone(),
            default_unicast_locator_list: Vec::new(),
            metatraffic_unicast_locator_list: Vec::new(),
            metatraffic_multicast_locator_list: Vec::new(),
            // chanel_message_sender: chanel_message_sender.clone(),
        };

        // let chanel_message_sender_clone = chanel_message_sender.clone();

        // std::thread::Builder::new()
        //     .name("SomethingOnMetatrafficMulticastSocket".to_string())
        //     .spawn(move || {
        //         let mut buf = [0; MAX_DATAGRAM_SIZE];
        //         loop {
        //             if let Ok(size) = metatraffic_multicast_socket.recv(&mut buf) {
        //                 if size > 0 {
        //                     chanel_message_sender_clone
        //                         .send(ChannelMessageKind::MetatrafficMulticastSocket(
        //                             buf[..size].into(),
        //                         ))
        //                         .expect("chanel_message sender alive");
        //                 }
        //             }
        //         }
        //     })
        //     .expect("failed to spawn thread");

        // let chanel_message_sender_clone = chanel_message_sender.clone();
        // std::thread::Builder::new()
        //     .name("SomethingOnMetatrafficUnicastSocket".to_string())
        //     .spawn(move || {
        //         let mut buf = [0; MAX_DATAGRAM_SIZE];
        //         loop {
        //             if let Ok(size) = metatraffic_unicast_socket.recv(&mut buf) {
        //                 if size > 0 {
        //                     chanel_message_sender_clone
        //                         .send(ChannelMessageKind::MetatrafficUnicastSocket(
        //                             buf[..size].into(),
        //                         ))
        //                         .expect("chanel_message sender alive");
        //                 }
        //             }
        //         }
        //     })
        //     .expect("failed to spawn thread");

        // let chanel_message_sender_clone = chanel_message_sender.clone();
        // std::thread::Builder::new()
        //     .name("SomethingOnDefaultUnicastSocket".to_string())
        //     .spawn(move || {
        //         let mut buf = [0; MAX_DATAGRAM_SIZE];
        //         loop {
        //             if let Ok(size) = default_unicast_socket.recv(&mut buf) {
        //                 if size > 0 {
        //                     chanel_message_sender_clone
        //                         .send(ChannelMessageKind::DefaultUnicastSocket(buf[..size].into()))
        //                         .expect("chanel_message sender alive");
        //                 }
        //             }
        //         }
        //     })
        //     .expect("failed to spawn thread");

        // let chanel_message_sender_clone = chanel_message_sender.clone();
        // std::thread::Builder::new()
        //     .name("Regular poke".to_string())
        //     .spawn(move || loop {
        //         std::thread::sleep(std::time::Duration::from_millis(50));
        //         chanel_message_sender_clone
        //             .send(ChannelMessageKind::Poke)
        //             .expect("chanel_message sender alive");
        //     })
        //     .expect("failed to spawn thread");

        // std::thread::Builder::new()
        //     .name("Socket receiver".to_string())
        //     .spawn(move || -> ! {
        //         let mut stateless_reader_list = Vec::new();
        //         let mut stateful_reader_list = Vec::new();
        //         let mut stateful_writer_list = Vec::new();
        //         loop {
        //             if let Ok(chanel_message) = chanel_message_receiver.recv() {
        //                 match chanel_message {
        //                     ChannelMessageKind::AddStatelessReader(stateless_reader) => {
        //                         stateless_reader_list.push(stateless_reader)
        //                     }
        //                     ChannelMessageKind::AddStatefulReader(stateful_reader) => {
        //                         stateful_reader_list.push(stateful_reader)
        //                     }
        //                     ChannelMessageKind::AddStatefulWriter(stateful_writer) => {
        //                         stateful_writer_list.push(stateful_writer)
        //                     }
        //                     ChannelMessageKind::MetatrafficMulticastSocket(datagram) => {
        //                         block_on(async {
        //                             process_message(
        //                                 &datagram,
        //                                 &message_writer,
        //                                 &RtpsChannelTransportClock,
        //                                 &mut stateless_reader_list,
        //                                 &stateful_reader_list,
        //                                 &stateful_writer_list,
        //                             )
        //                             .await
        //                         });
        //                     }
        //                     ChannelMessageKind::MetatrafficUnicastSocket(datagram) => {
        //                         block_on(async {
        //                             process_message(
        //                                 &datagram,
        //                                 &message_writer,
        //                                 &RtpsChannelTransportClock,
        //                                 &mut stateless_reader_list,
        //                                 &stateful_reader_list,
        //                                 &stateful_writer_list,
        //                             )
        //                             .await
        //                         });
        //                     }
        //                     ChannelMessageKind::DefaultUnicastSocket(datagram) => {
        //                         block_on(async {
        //                             process_message(
        //                                 &datagram,
        //                                 &message_writer,
        //                                 &RtpsChannelTransportClock,
        //                                 &mut stateless_reader_list,
        //                                 &stateful_reader_list,
        //                                 &stateful_writer_list,
        //                             )
        //                             .await
        //                         });
        //                     }
        //                     ChannelMessageKind::Poke => block_on(async {
        //                         for rtps_stateful_writer in &stateful_writer_list {
        //                             rtps_stateful_writer
        //                                 .lock()
        //                                 .await
        //                                 .write_message(
        //                                     message_writer.as_ref(),
        //                                     &RtpsChannelTransportClock,
        //                                 )
        //                                 .await;
        //                         }
        //                     }),
        //                 }
        //             }
        //         }
        //     })
        //     .expect("failed to spawn thread");

        Box::new(participant)
    }
}

async fn process_message(
    datagram: &[u8],
    message_writer: &MessageWriter,
    clock: &impl Clock,
    stateless_reader_list: &mut [RtpsStatelessReader],
    stateful_reader_list: &[Arc<Mutex<RtpsStatefulReader>>],
    stateful_writer_list: &[Arc<Mutex<RtpsStatefulWriter>>],
) {
    for stateless_reader in stateless_reader_list {
        stateless_reader.process_message(datagram).await.ok();
    }
    for stateful_reader in stateful_reader_list {
        stateful_reader
            .lock()
            .await
            .process_message(datagram, message_writer)
            .await
            .ok();
    }
    for stateful_writer in stateful_writer_list {
        stateful_writer
            .lock()
            .await
            .process_message(datagram, message_writer, clock)
            .await
            .ok();
    }
}
struct MessageWriter {
    guid_prefix: GuidPrefix,
}

impl MessageWriter {
    fn new(guid_prefix: GuidPrefix) -> Self {
        Self { guid_prefix }
    }
}
impl WriteMessage for MessageWriter {
    async fn write_message(&self, datagram: &[u8], locator_list: &[Locator]) {
        // for &destination_locator in locator_list {
        //     if UdpLocator(destination_locator).is_multicast() {
        //         let socket2: socket2::Socket = self.socket.try_clone().unwrap().into();
        //         let interface_addresses = NetworkInterface::show();
        //         let interface_addresses: Vec<_> = interface_addresses
        //             .expect("Could not scan interfaces")
        //             .into_iter()
        //             .flat_map(|i| {
        //                 i.addr.into_iter().filter_map(|a| match a {
        //                     Addr::V4(v4) => Some(v4.ip),
        //                     _ => None,
        //                 })
        //             })
        //             .collect();
        //         for address in interface_addresses {
        //             if socket2.set_multicast_if_v4(&address).is_ok() {
        //                 self.socket
        //                     .send_to(datagram, UdpLocator(destination_locator))
        //                     .ok();
        //             }
        //         }
        //     } else {
        //         self.socket
        //             .send_to(datagram, UdpLocator(destination_locator))
        //             .ok();
        //     }
        // }
    }

    fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
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

pub struct RtpsChannelTransportParticipant {
    guid: Guid,
    message_writer: Arc<MessageWriter>,
    default_unicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    // chanel_message_sender: Sender<ChannelMessageKind>,
}

impl TransportParticipant for RtpsChannelTransportParticipant {
    type HistoryCache = Box<dyn HistoryCache>;
    type StatelessReader = Box<dyn TransportStatelessReader>;
    type StatelessWriter = Box<dyn TransportStatelessWriter>;
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
        &self.metatraffic_unicast_locator_list
    }
    fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_multicast_locator_list
    }
    fn default_unicast_locator_list(&self) -> &[Locator] {
        &self.default_unicast_locator_list
    }
    fn default_multicast_locator_list(&self) -> &[Locator] {
        &[]
    }
    fn create_stateless_reader(
        &mut self,
        entity_id: EntityId,
        reader_history_cache: Self::HistoryCache,
    ) -> Self::StatelessReader {
        struct StatelessReader {
            guid: Guid,
        }
        impl TransportStatelessReader for StatelessReader {
            fn guid(&self) -> Guid {
                self.guid
            }
        }
        let guid = Guid::new(self.guid.prefix(), entity_id);
        // self.chanel_message_sender
        //     .send(ChannelMessageKind::AddStatelessReader(
        //         RtpsStatelessReader::new(guid, reader_history_cache),
        //     ))
        //     .expect("chanel_message receiver alive");
        Box::new(StatelessReader {
            guid: Guid::new(self.guid.prefix(), entity_id),
        })
    }
    fn create_stateless_writer(&mut self, entity_id: EntityId) -> Self::StatelessWriter {
        struct StatelessWriter {
            rtps_writer: RtpsStatelessWriter,
            message_writer: Arc<MessageWriter>,
        }
        impl TransportStatelessWriter for StatelessWriter {
            fn guid(&self) -> Guid {
                self.rtps_writer.guid()
            }
            fn history_cache(&mut self) -> &mut dyn HistoryCache {
                self
            }

            fn add_reader_locator(&mut self, locator: Locator) {
                self.rtps_writer.reader_locator_add(locator);
            }

            fn remove_reader_locator(&mut self, locator: &Locator) {
                self.rtps_writer.reader_locator_remove(*locator);
            }
        }
        impl HistoryCache for StatelessWriter {
            fn add_change(
                &mut self,
                cache_change: CacheChange,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                self.rtps_writer.add_change(cache_change);
                let message_writer = self.message_writer.clone();
                block_on(async {
                    self.rtps_writer
                        .write_message(message_writer.as_ref())
                        .await
                });
                Box::pin(async {})
            }

            fn remove_change(
                &mut self,
                sequence_number: i64,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                self.rtps_writer.remove_change(sequence_number);
                Box::pin(async {})
            }
        }
        let guid = Guid::new(self.guid.prefix(), entity_id);
        Box::new(StatelessWriter {
            rtps_writer: RtpsStatelessWriter::new(guid),
            message_writer: self.message_writer.clone(),
        })
    }

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        reliability_kind: ReliabilityKind,
        reader_history_cache: Self::HistoryCache,
    ) -> Self::StatefulReader {
        struct StatefulReader {
            guid: Guid,
            rtps_stateful_reader: Arc<Mutex<RtpsStatefulReader>>,
        }
        impl TransportStatefulReader for StatefulReader {
            fn guid(&self) -> Guid {
                self.guid
            }
            fn is_historical_data_received(&self) -> bool {
                block_on(async {
                    self.rtps_stateful_reader
                        .lock()
                        .await
                        .is_historical_data_received()
                })
            }
            fn add_matched_writer(&mut self, writer_proxy: WriterProxy) {
                block_on(async {
                    self.rtps_stateful_reader
                        .lock()
                        .await
                        .add_matched_writer(&writer_proxy)
                })
            }
            fn remove_matched_writer(&mut self, remote_writer_guid: Guid) {
                block_on(async {
                    self.rtps_stateful_reader
                        .lock()
                        .await
                        .delete_matched_writer(remote_writer_guid)
                })
            }
        }

        let guid = Guid::new(self.guid.prefix(), entity_id);
        let rtps_stateful_reader = Arc::new(Mutex::new(RtpsStatefulReader::new(
            guid,
            reader_history_cache,
            reliability_kind,
        )));
        // self.chanel_message_sender
        //     .send(ChannelMessageKind::AddStatefulReader(
        //         rtps_stateful_reader.clone(),
        //     ))
        //     .expect("chanel_message receiver alive");
        Box::new(StatefulReader {
            guid,
            rtps_stateful_reader,
        })
    }

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        _reliability_kind: ReliabilityKind,
    ) -> Self::StatefulWriter {
        struct StatefulWriter {
            guid: Guid,
            rtps_stateful_writer: Arc<Mutex<RtpsStatefulWriter>>,
            message_writer: Arc<MessageWriter>,
            default_unicast_locator_list: Vec<Locator>,
        }
        impl TransportStatefulWriter for StatefulWriter {
            fn guid(&self) -> Guid {
                self.guid
            }
            fn history_cache(&mut self) -> &mut dyn HistoryCache {
                self
            }
            fn is_change_acknowledged(&self, sequence_number: i64) -> bool {
                block_on(async {
                    self.rtps_stateful_writer
                        .lock()
                        .await
                        .is_change_acknowledged(sequence_number)
                })
            }
            fn add_matched_reader(&mut self, mut reader_proxy: ReaderProxy) {
                if reader_proxy.unicast_locator_list.is_empty() {
                    reader_proxy
                        .unicast_locator_list
                        .clone_from(&self.default_unicast_locator_list);
                }
                block_on(async {
                    self.rtps_stateful_writer
                        .lock()
                        .await
                        .add_matched_reader(&reader_proxy);
                })
            }
            fn remove_matched_reader(&mut self, remote_reader_guid: Guid) {
                block_on(async {
                    self.rtps_stateful_writer
                        .lock()
                        .await
                        .delete_matched_reader(remote_reader_guid);
                })
            }
        }
        impl HistoryCache for StatefulWriter {
            fn add_change(
                &mut self,
                cache_change: CacheChange,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                let rtps_stateful_writer = self.rtps_stateful_writer.clone();
                let message_writer = self.message_writer.clone();
                Box::pin(async move {
                    rtps_stateful_writer.lock().await.add_change(cache_change);
                    rtps_stateful_writer
                        .lock()
                        .await
                        .write_message(message_writer.as_ref(), &RtpsChannelTransportClock)
                        .await;
                })
            }

            fn remove_change(
                &mut self,
                sequence_number: i64,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                let rtps_stateful_writer = self.rtps_stateful_writer.clone();
                Box::pin(async move {
                    rtps_stateful_writer
                        .lock()
                        .await
                        .remove_change(sequence_number);
                })
            }
        }

        let guid = Guid::new(self.guid.prefix(), entity_id);
        let rtps_stateful_writer = Arc::new(Mutex::new(RtpsStatefulWriter::new(guid, 1024)));
        // self.chanel_message_sender
        //     .send(ChannelMessageKind::AddStatefulWriter(
        //         rtps_stateful_writer.clone(),
        //     ))
        //     .expect("chanel_message receiver alive");
        Box::new(StatefulWriter {
            guid,
            rtps_stateful_writer,
            message_writer: self.message_writer.clone(),
            default_unicast_locator_list: self.default_unicast_locator_list.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::types::{DurabilityKind, ENTITYID_UNKNOWN};
    use dust_dds::transport::types::ChangeKind;
    use std::sync::mpsc::{sync_channel, SyncSender};

    #[test]
    fn basic_transport_stateful_reader_writer_usage() {
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
        let reliability_kind = ReliabilityKind::BestEffort;
        let (sender, receiver) = sync_channel(0);
        let reader_history_cache = Box::new(MockHistoryCache(sender));
        let mut reader =
            participant.create_stateful_reader(entity_id, reliability_kind, reader_history_cache);

        let entity_id = EntityId::new([5, 6, 7], 8);
        let mut writer = participant.create_stateful_writer(entity_id, reliability_kind);

        let reader_proxy = ReaderProxy {
            remote_reader_guid: reader.guid(),
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

    //     #[test]
    //     fn basic_transport_stateless_reader_writer_usage() {
    //         let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    //         let domain_id = 0;
    //         let transport = RtpsChannelTransportParticipantFactoryBuilder::new()
    //             .build()
    //             .unwrap();
    //         let mut participant = transport.create_participant(guid_prefix, domain_id);

    //         struct MockHistoryCache(SyncSender<CacheChange>);

    //         impl HistoryCache for MockHistoryCache {
    //             fn add_change(
    //                 &mut self,
    //                 cache_change: CacheChange,
    //             ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    //                 self.0.send(cache_change).unwrap();
    //                 Box::pin(async {})
    //             }
    //             fn remove_change(
    //                 &mut self,
    //                 _sequence_number: i64,
    //             ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    //                 Box::pin(async {
    //                     unimplemented!();
    //                 })
    //             }
    //         }

    //         let entity_id = EntityId::new([1, 2, 3], 4);
    //         let (sender, receiver) = sync_channel(0);
    //         let reader_history_cache = Box::new(MockHistoryCache(sender));
    //         let _reader = participant.create_stateless_reader(entity_id, reader_history_cache);

    //         let entity_id = EntityId::new([5, 6, 7], 8);
    //         let mut writer = participant.create_stateless_writer(entity_id);
    //         for locator in participant.default_unicast_locator_list() {
    //             writer.add_reader_locator(locator.clone());
    //         }

    //         let cache_change = CacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: writer.guid(),
    //             sequence_number: 1,
    //             source_timestamp: None,
    //             instance_handle: None,
    //             data_value: vec![0, 0, 0, 0, 1, 2, 3, 4].into(),
    //         };
    //         writer.history_cache().add_change(cache_change.clone());

    //         let received_cache_change = receiver
    //             .recv_timeout(std::time::Duration::from_secs(30))
    //             .unwrap();
    //         assert_eq!(cache_change, received_cache_change);
    //     }
}
