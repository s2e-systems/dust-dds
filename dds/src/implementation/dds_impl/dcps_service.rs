use std::{
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    sync::{
        atomic::{self, AtomicBool},
        mpsc::{Receiver, SyncSender},
        Arc,
    },
    thread::JoinHandle,
};

use crate::{
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{GapSubmessage, InfoDestinationSubmessage, InfoTimestampSubmessage},
                types::ProtocolId,
                RtpsMessage, RtpsSubmessageKind,
            },
            reader_proxy::WriterAssociatedReaderProxy,
            transport::TransportWrite,
            types::{EntityId, GuidPrefix, ReliabilityKind, SequenceNumber},
        },
        rtps_udp_psm::udp_transport::UdpTransport,
        utils::{condvar::DdsCondvar, shared_object::DdsShared},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        time::{Duration, DurationKind, Time},
    },
    topic_definition::type_support::{DdsSerialize, DdsSerializedKey, DdsType, LittleEndian},
};

use super::{
    domain_participant_impl::{AnnounceKind, DomainParticipantImpl},
    user_defined_data_writer::UserDefinedDataWriter,
};

pub struct DcpsService {
    participant: DdsShared<DomainParticipantImpl>,
    quit: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
    sedp_condvar: DdsCondvar,
    user_defined_data_send_condvar: DdsCondvar,
    sender_socket: UdpSocket,
    announce_sender: SyncSender<AnnounceKind>,
}

impl DcpsService {
    pub fn new(
        participant: DdsShared<DomainParticipantImpl>,
        mut metatraffic_multicast_transport: UdpTransport,
        mut metatraffic_unicast_transport: UdpTransport,
        mut default_unicast_transport: UdpTransport,
        announce_sender: SyncSender<AnnounceKind>,
        announce_receiver: Receiver<AnnounceKind>,
    ) -> DdsResult<Self> {
        let quit = Arc::new(AtomicBool::new(false));
        let mut threads = Vec::new();

        // //////////// Notification thread
        {
            let domain_participant = participant.clone();
            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                domain_participant.update_communication_status().ok();
                std::thread::sleep(std::time::Duration::from_millis(50));
            }));
        }

        // //////////// SPDP Communication

        // ////////////// SPDP participant discovery
        {
            let domain_participant = participant.clone();

            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) = metatraffic_multicast_transport.read() {
                    domain_participant
                        .receive_built_in_data(locator, message)
                        .ok();
                }
            }));
        }

        //  ////////////// Entity announcer thread
        {
            let domain_participant = participant.clone();

            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Ok(announce_kind) = announce_receiver.recv() {
                    match announce_kind {
                        AnnounceKind::CreatedDataReader(discovered_reader_data) => {
                            announce_created_data_reader(
                                &domain_participant,
                                discovered_reader_data,
                            )
                        }
                        AnnounceKind::CreatedDataWriter(discovered_writer_data) => {
                            announce_created_data_writer(
                                &domain_participant,
                                discovered_writer_data,
                            )
                        }
                        AnnounceKind::CratedTopic(discovered_topic_data) => {
                            announce_created_topic(&domain_participant, discovered_topic_data)
                        }
                        AnnounceKind::DeletedDataReader(deleted_reader_handle) => {
                            announce_deleted_reader(&domain_participant, deleted_reader_handle)
                        }
                        AnnounceKind::DeletedDataWriter(deleted_writer_handle) => {
                            announce_deleted_writer(&domain_participant, deleted_writer_handle)
                        }
                        AnnounceKind::DeletedParticipant => (),
                    }
                }
            }));
        }

        // //////////// Unicast metatraffic Communication receive
        {
            let domain_participant = participant.clone();

            let task_quit = quit.clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) = metatraffic_unicast_transport.read() {
                    domain_participant
                        .receive_built_in_data(locator, message)
                        .ok();
                }
            }));
        }

        // //////////// Unicast metatraffic Communication send
        {
            let domain_participant = participant.clone();
            let socket = UdpSocket::bind("0.0.0.0:0000").unwrap();

            let mut metatraffic_unicast_transport_send = UdpTransport::new(socket);
            let task_quit = quit.clone();
            let sedp_condvar_clone = domain_participant.sedp_condvar().clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }
                let _r = sedp_condvar_clone.wait_timeout(Duration::new(0, 500000000));

                let header = RtpsMessageHeader {
                    protocol: ProtocolId::PROTOCOL_RTPS,
                    version: domain_participant.protocol_version(),
                    vendor_id: domain_participant.vendor_id(),
                    guid_prefix: domain_participant.guid().prefix(),
                };

                let now = domain_participant.get_current_time();

                domain_participant
                    .get_builtin_publisher()
                    .spdp_builtin_participant_writer()
                    .send_message(header, &mut metatraffic_unicast_transport_send);
                domain_participant
                    .get_builtin_publisher()
                    .sedp_builtin_publications_writer()
                    .send_message(header, &mut metatraffic_unicast_transport_send, now);
                domain_participant
                    .get_builtin_publisher()
                    .sedp_builtin_subscriptions_writer()
                    .send_message(header, &mut metatraffic_unicast_transport_send, now);
                domain_participant
                    .get_builtin_publisher()
                    .sedp_builtin_topics_writer()
                    .send_message(header, &mut metatraffic_unicast_transport_send, now);

                domain_participant
                    .get_builtin_subscriber()
                    .send_message(header, &mut metatraffic_unicast_transport_send);
            }));
        }

        // //////////// User-defined Communication receive
        {
            let domain_participant = participant.clone();
            let task_quit = quit.clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) = default_unicast_transport.read() {
                    domain_participant
                        .receive_user_defined_data(locator, message)
                        .ok();
                }
            }));
        }

        // //////////// User-defined Communication send
        {
            let domain_participant = participant.clone();
            let socket = UdpSocket::bind("0.0.0.0:0000").unwrap();
            let mut default_unicast_transport_send = UdpTransport::new(socket);
            let task_quit = quit.clone();
            let user_defined_data_send_condvar_clone =
                domain_participant.user_defined_data_send_condvar().clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                let _r = user_defined_data_send_condvar_clone
                    .wait_timeout(Duration::new(0, 100_000_000));

                let header = RtpsMessageHeader {
                    protocol: ProtocolId::PROTOCOL_RTPS,
                    version: domain_participant.protocol_version(),
                    vendor_id: domain_participant.vendor_id(),
                    guid_prefix: domain_participant.guid().prefix(),
                };
                let now = domain_participant.get_current_time();

                for publisher in domain_participant.publisher_list() {
                    for data_writer in publisher.data_writer_list() {
                        let writer_id = data_writer.guid().entity_id();
                        let data_max_size_serialized = data_writer.data_max_size_serialized();
                        let heartbeat_period = data_writer.heartbeat_period();
                        let first_sn = SequenceNumber::new(1);
                        // data_writer.change_list() writer_cache
                        // .get_seq_num_min()
                        // .unwrap_or(SequenceNumber::new(1));
                        let last_sn = SequenceNumber::new(0);
                        // writer_cache
                        //     .get_seq_num_max()
                        //     .unwrap_or_else(|| SequenceNumber::new(0)),
                        remove_stale_writer_changes(&data_writer, now);
                        for mut reader_proxy in &mut data_writer.matched_reader_list() {
                            match reader_proxy.reliability() {
                                ReliabilityKind::BestEffort => {
                                    send_message_best_effort_reader_proxy(
                                        &mut reader_proxy,
                                        data_max_size_serialized,
                                        header,
                                        &mut default_unicast_transport_send,
                                    )
                                }
                                ReliabilityKind::Reliable => send_message_reliable_reader_proxy(
                                    &mut reader_proxy,
                                    data_max_size_serialized,
                                    header,
                                    &mut default_unicast_transport_send,
                                    writer_id,
                                    first_sn,
                                    last_sn,
                                    heartbeat_period,
                                ),
                            }
                        }
                    }
                }

                for subscriber in domain_participant.subscriber_list() {
                    for data_reader in subscriber.data_reader_list() {
                        data_reader.send_message(header, &mut default_unicast_transport_send)
                    }
                }
            }));
        }

        let sender_socket = UdpSocket::bind("0.0.0.0:0000").unwrap();

        let sedp_condvar = participant.sedp_condvar().clone();
        let user_defined_data_send_condvar = participant.user_defined_data_send_condvar().clone();
        Ok(DcpsService {
            participant,
            quit,
            threads,
            sedp_condvar,
            user_defined_data_send_condvar,
            sender_socket,
            announce_sender,
        })
    }

    pub fn participant(&self) -> &DdsShared<DomainParticipantImpl> {
        &self.participant
    }

    pub fn shutdown_tasks(&mut self) {
        self.quit.store(true, atomic::Ordering::SeqCst);

        self.sedp_condvar.notify_all();
        self.user_defined_data_send_condvar.notify_all();
        self.announce_sender
            .send(AnnounceKind::DeletedParticipant)
            .ok();

        if let Some(default_unicast_locator) =
            self.participant.default_unicast_locator_list().get(0)
        {
            let port: u32 = default_unicast_locator.port().into();
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port as u16);
            self.sender_socket.send_to(&[0], addr).ok();
        }

        if let Some(metatraffic_unicast_locator) =
            self.participant.metatraffic_unicast_locator_list().get(0)
        {
            let port: u32 = metatraffic_unicast_locator.port().into();
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port as u16);
            self.sender_socket.send_to(&[0], addr).ok();
        }

        if let Some(metatraffic_multicast_transport) =
            self.participant.metatraffic_multicast_locator_list().get(0)
        {
            let addr: [u8; 16] = metatraffic_multicast_transport.address().into();
            let port: u32 = metatraffic_multicast_transport.port().into();
            let addr = SocketAddrV4::new(
                Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                port as u16,
            );
            self.sender_socket.send_to(&[0], addr).ok();
        }

        while let Some(thread) = self.threads.pop() {
            thread.join().unwrap();
        }
    }
}

fn announce_created_data_reader(
    domain_participant: &DomainParticipantImpl,
    discovered_reader_data: DiscoveredReaderData,
) {
    let reader_data = &DiscoveredReaderData {
        reader_proxy: ReaderProxy {
            unicast_locator_list: domain_participant.default_unicast_locator_list().to_vec(),
            multicast_locator_list: domain_participant.default_multicast_locator_list().to_vec(),
            ..discovered_reader_data.reader_proxy
        },
        ..discovered_reader_data
    };

    let mut serialized_data = Vec::new();
    reader_data
        .serialize::<_, LittleEndian>(&mut serialized_data)
        .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();
    domain_participant
        .get_builtin_publisher()
        .sedp_builtin_subscriptions_writer()
        .write_w_timestamp(
            serialized_data,
            reader_data.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_created_data_writer(
    domain_participant: &DomainParticipantImpl,
    discovered_writer_data: DiscoveredWriterData,
) {
    let writer_data = &DiscoveredWriterData {
        writer_proxy: WriterProxy {
            unicast_locator_list: domain_participant.default_unicast_locator_list().to_vec(),
            multicast_locator_list: domain_participant.default_multicast_locator_list().to_vec(),
            ..discovered_writer_data.writer_proxy
        },
        ..discovered_writer_data
    };

    let mut serialized_data = Vec::new();
    writer_data
        .serialize::<_, LittleEndian>(&mut serialized_data)
        .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher()
        .sedp_builtin_publications_writer()
        .write_w_timestamp(
            serialized_data,
            writer_data.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_created_topic(
    domain_participant: &DomainParticipantImpl,
    discovered_topic: DiscoveredTopicData,
) {
    let mut serialized_data = Vec::new();
    discovered_topic
        .serialize::<_, LittleEndian>(&mut serialized_data)
        .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher()
        .sedp_builtin_topics_writer()
        .write_w_timestamp(
            serialized_data,
            discovered_topic.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_deleted_reader(
    domain_participant: &DomainParticipantImpl,
    reader_handle: InstanceHandle,
) {
    let serialized_key = DdsSerializedKey::from(reader_handle.as_ref());
    let instance_serialized_key =
        cdr::serialize::<_, _, cdr::CdrLe>(&serialized_key, cdr::Infinite)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
            .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher()
        .sedp_builtin_subscriptions_writer()
        .dispose_w_timestamp(instance_serialized_key, reader_handle, timestamp)
        .expect("Should not fail to write built-in message");
}

fn announce_deleted_writer(
    domain_participant: &DomainParticipantImpl,
    writer_handle: InstanceHandle,
) {
    let serialized_key = DdsSerializedKey::from(writer_handle.as_ref());
    let instance_serialized_key =
        cdr::serialize::<_, _, cdr::CdrLe>(&serialized_key, cdr::Infinite)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
            .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher()
        .sedp_builtin_publications_writer()
        .dispose_w_timestamp(instance_serialized_key, writer_handle, timestamp)
        .expect("Should not fail to write built-in message");
}

fn remove_stale_writer_changes(writer: &UserDefinedDataWriter, now: Time) {
    let timespan_duration = writer.get_qos().lifespan.duration;
    writer.remove_change(|cc| DurationKind::Finite(now - cc.timestamp()) > timespan_duration);
}

fn send_message_best_effort_reader_proxy(
    reader_proxy: &mut WriterAssociatedReaderProxy,
    data_max_size_serialized: usize,
    header: RtpsMessageHeader,
    transport: &mut impl TransportWrite,
) {
    let info_dst = info_destination_submessage(reader_proxy.remote_reader_guid().prefix());
    let mut submessages = vec![info_dst];

    while !reader_proxy.unsent_changes().is_empty() {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4
        let reader_id = reader_proxy.remote_reader_guid().entity_id();
        let change = reader_proxy.next_unsent_change();

        if change.is_relevant() {
            let timestamp = change.timestamp();

            if change.data_value().len() > data_max_size_serialized {
                let data_frag_submessage_list = change
                    .cache_change()
                    .as_data_frag_submessages(data_max_size_serialized, reader_id);
                for data_frag_submessage in data_frag_submessage_list {
                    let info_dst =
                        info_destination_submessage(reader_proxy.remote_reader_guid().prefix());

                    let into_timestamp = info_timestamp_submessage(timestamp);
                    let data_frag = RtpsSubmessageKind::DataFrag(data_frag_submessage);

                    let submessages = vec![info_dst, into_timestamp, data_frag];

                    transport.write(
                        &RtpsMessage::new(header, submessages),
                        reader_proxy.unicast_locator_list(),
                    )
                }
            } else {
                submessages.push(info_timestamp_submessage(timestamp));
                submessages.push(RtpsSubmessageKind::Data(
                    change.cache_change().as_data_submessage(reader_id),
                ))
            }
        } else {
            let gap_submessage: GapSubmessage = change
                .cache_change()
                .as_gap_message(reader_proxy.remote_reader_guid().entity_id());
            submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
        }
    }

    // Send messages only if more than INFO_DST is added
    if submessages.len() > 1 {
        transport.write(
            &RtpsMessage::new(header, submessages),
            reader_proxy.unicast_locator_list(),
        )
    }
}

fn send_message_reliable_reader_proxy(
    reader_proxy: &mut WriterAssociatedReaderProxy,
    data_max_size_serialized: usize,
    header: RtpsMessageHeader,
    transport: &mut impl TransportWrite,
    writer_id: EntityId,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    heartbeat_period: Duration,
) {
    let reader_id = reader_proxy.remote_reader_guid().entity_id();

    let info_dst = info_destination_submessage(reader_proxy.remote_reader_guid().prefix());

    let mut submessages = vec![info_dst];

    // Top part of the state machine - Figure 8.19 RTPS standard
    if !reader_proxy.unsent_changes().is_empty() {
        // Note: The readerId is set to the remote reader ID as described in 8.4.9.2.12 Transition T12
        // in confront to ENTITYID_UNKNOWN as described in 8.4.9.2.4 Transition T4

        while !reader_proxy.unsent_changes().is_empty() {
            let change = reader_proxy.next_unsent_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // "( a_change BELONGS-TO the_reader_proxy.unsent_changes() ) == FALSE"
            // should be full-filled by next_unsent_change()
            if change.is_relevant() {
                if change.data_value().len() > data_max_size_serialized {
                    todo!();
                    // directly_send_data_frag(
                    //     change.cache_change(),
                    //     writer_cache,
                    //     writer_id,
                    //     data_max_size_serialized,
                    //     header,
                    //     transport,
                    // );
                    return;
                } else {
                    submessages.push(info_timestamp_submessage(change.timestamp()));
                    submessages.push(RtpsSubmessageKind::Data(
                        change.cache_change().as_data_submessage(reader_id),
                    ))
                }
            } else {
                let gap_submessage: GapSubmessage = change.cache_change().as_gap_message(reader_id);

                submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
            }
        }

        let heartbeat = reader_proxy
            .heartbeat_machine()
            .submessage(writer_id, first_sn, last_sn);
        submessages.push(heartbeat);
    } else if reader_proxy.unacked_changes().is_empty() {
        // Idle
    } else if reader_proxy
        .heartbeat_machine()
        .is_time_for_heartbeat(heartbeat_period)
    {
        let heartbeat = reader_proxy
            .heartbeat_machine()
            .submessage(writer_id, first_sn, last_sn);
        submessages.push(heartbeat);
    }

    // Middle-part of the state-machine - Figure 8.19 RTPS standard
    if !reader_proxy.requested_changes().is_empty() {
        let reader_id = reader_proxy.remote_reader_guid().entity_id();

        while !reader_proxy.requested_changes().is_empty() {
            let change_for_reader = reader_proxy.next_requested_change();
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
            // should be full-filled by next_requested_change()
            if change_for_reader.is_relevant() {
                if change_for_reader.data_value().len() > data_max_size_serialized {
                    todo!();
                    // directly_send_data_frag(
                    //     change_for_reader.cache_change(),
                    //     writer_cache,
                    //     writer_id,
                    //     data_max_size_serialized,
                    //     header,
                    //     transport,
                    // );
                    return;
                } else {
                    submessages.push(info_timestamp_submessage(change_for_reader.timestamp()));
                    submessages.push(RtpsSubmessageKind::Data(
                        change_for_reader
                            .cache_change()
                            .as_data_submessage(reader_id),
                    ))
                }
            } else {
                let gap_submessage: GapSubmessage =
                    change_for_reader.cache_change().as_gap_message(reader_id);

                submessages.push(RtpsSubmessageKind::Gap(gap_submessage));
            }
        }
        let heartbeat = reader_proxy
            .heartbeat_machine()
            .submessage(writer_id, first_sn, last_sn);
        submessages.push(heartbeat);
    }
    // Send messages only if more or equal than INFO_DST and HEARTBEAT is added
    if submessages.len() >= 2 {
        transport.write(
            &RtpsMessage::new(header, submessages),
            reader_proxy.unicast_locator_list(),
        )
    }
}

fn info_timestamp_submessage<'a>(timestamp: Time) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::InfoTimestamp(InfoTimestampSubmessage {
        endianness_flag: true,
        invalidate_flag: false,
        timestamp: crate::implementation::rtps::messages::types::Time::new(
            timestamp.sec(),
            timestamp.nanosec(),
        ),
    })
}
fn info_destination_submessage<'a>(guid_prefix: GuidPrefix) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::InfoDestination(InfoDestinationSubmessage {
        endianness_flag: true,
        guid_prefix,
    })
}

fn directly_send_data_frag(// &mut self,
    // cache_change: &RtpsWriterCacheChange,
    // writer_cache: &WriterHistoryCache,
    // writer_id: EntityId,
    // data_max_size_serialized: usize,
    // header: RtpsMessageHeader,
    // transport: &mut impl TransportWrite,
) {
    // let reader_id = self.remote_reader_guid().entity_id();
    // let timestamp = cache_change.timestamp();

    // let mut data_frag_submessage_list = cache_change
    //     .as_data_frag_submessages(data_max_size_serialized, reader_id)
    //     .into_iter()
    //     .peekable();

    // while let Some(data_frag_submessage) = data_frag_submessage_list.next() {
    //     let writer_sn = data_frag_submessage.writer_sn;
    //     let last_fragment_num = FragmentNumber::new(
    //         u32::from(data_frag_submessage.fragment_starting_num)
    //             + u16::from(data_frag_submessage.fragments_in_submessage) as u32
    //             - 1,
    //     );

    //     let info_dst = info_destination_submessage(self.remote_reader_guid().prefix());
    //     let into_timestamp = info_timestamp_submessage(timestamp);
    //     let data_frag = RtpsSubmessageKind::DataFrag(data_frag_submessage);

    //     let is_last_fragment = data_frag_submessage_list.peek().is_none();
    //     let submessages = if is_last_fragment {
    //         let heartbeat_frag =
    //             self.heartbeat_frag_machine
    //                 .submessage(writer_id, writer_sn, last_fragment_num);
    //         vec![info_dst, into_timestamp, data_frag, heartbeat_frag]
    //     } else {
    //         let heartbeat = self.heartbeat_machine.submessage(writer_id, writer_cache);
    //         vec![info_dst, into_timestamp, data_frag, heartbeat]
    //     };
    //     transport.write(
    //         &RtpsMessage::new(header, submessages),
    //         self.unicast_locator_list(),
    //     )
    // }
}
