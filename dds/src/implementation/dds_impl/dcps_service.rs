use std::{
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    sync::{
        atomic::{self, AtomicBool},
        mpsc::{Receiver, SyncSender},
        Arc,
    },
    thread::JoinHandle,
};

use fnmatch_regex::glob_to_regex;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        rtps::{
            history_cache::RtpsWriterCacheChange,
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{GapSubmessage, InfoDestinationSubmessage, InfoTimestampSubmessage},
                types::{FragmentNumber, ProtocolId},
                RtpsMessage, RtpsSubmessageKind,
            },
            reader_proxy::WriterAssociatedReaderProxy,
            transport::TransportWrite,
            types::{
                EntityId, Guid, GuidPrefix, Locator, ReliabilityKind, SequenceNumber,
                ENTITYID_PARTICIPANT,
            },
        },
        rtps_udp_psm::udp_transport::UdpTransport,
        utils::{condvar::DdsCondvar, shared_object::DdsShared},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        time::{Duration, DurationKind, Time},
    },
    subscription::sample_info::{
        InstanceStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE,
    },
    topic_definition::type_support::{DdsSerialize, DdsSerializedKey, DdsType, LittleEndian},
};

use super::{
    builtin_stateful_writer::BuiltinStatefulWriter,
    domain_participant_impl::{AnnounceKind, DomainParticipantImpl},
    message_receiver::MessageReceiver,
    status_listener::StatusListener,
    user_defined_data_writer::UserDefinedDataWriter,
    user_defined_subscriber::UserDefinedSubscriber,
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
                    MessageReceiver::new(domain_participant.get_current_time())
                        .process_message(
                            domain_participant.guid().prefix(),
                            core::slice::from_ref(&domain_participant.get_builtin_publisher()),
                            core::slice::from_ref(&domain_participant.get_builtin_subscriber()),
                            locator,
                            &message,
                            &mut domain_participant.get_status_listener_lock(),
                        )
                        .ok();

                    domain_participant.discover_matched_participants().ok();
                    domain_participant.discover_matched_readers().ok();
                    discover_matched_writers(&domain_participant).ok();
                    domain_participant.discover_matched_topics().ok();
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
                    MessageReceiver::new(domain_participant.get_current_time())
                        .process_message(
                            domain_participant.guid().prefix(),
                            core::slice::from_ref(&domain_participant.get_builtin_publisher()),
                            core::slice::from_ref(&domain_participant.get_builtin_subscriber()),
                            locator,
                            &message,
                            &mut domain_participant.get_status_listener_lock(),
                        )
                        .ok();

                    domain_participant.discover_matched_participants().ok();
                    domain_participant.discover_matched_readers().ok();
                    discover_matched_writers(&domain_participant).ok();
                    domain_participant.discover_matched_topics().ok();
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

                let _now = domain_participant.get_current_time();

                domain_participant
                    .get_builtin_publisher()
                    .spdp_builtin_participant_writer()
                    .send_message(header, &mut metatraffic_unicast_transport_send);

                builtin_stateful_writer_send_message(
                    domain_participant
                        .get_builtin_publisher()
                        .sedp_builtin_publications_writer(),
                    header,
                    &mut metatraffic_unicast_transport_send,
                );

                builtin_stateful_writer_send_message(
                    domain_participant
                        .get_builtin_publisher()
                        .sedp_builtin_subscriptions_writer(),
                    header,
                    &mut metatraffic_unicast_transport_send,
                );

                builtin_stateful_writer_send_message(
                    domain_participant
                        .get_builtin_publisher()
                        .sedp_builtin_topics_writer(),
                    header,
                    &mut metatraffic_unicast_transport_send,
                );

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

                for publisher in &domain_participant.user_defined_publisher_list() {
                    for data_writer in &publisher.data_writer_list() {
                        let writer_id = data_writer.guid().entity_id();
                        let data_max_size_serialized = data_writer.data_max_size_serialized();
                        let heartbeat_period = data_writer.heartbeat_period();
                        let first_sn = data_writer
                            .change_list()
                            .into_iter()
                            .map(|x| x.sequence_number())
                            .min()
                            .unwrap_or(SequenceNumber::new(1));
                        let last_sn = data_writer
                            .change_list()
                            .into_iter()
                            .map(|x| x.sequence_number())
                            .max()
                            .unwrap_or_else(|| SequenceNumber::new(0));
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

                for subscriber in &domain_participant.user_defined_subscriber_list() {
                    for data_reader in &subscriber.data_reader_list() {
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
            let cache_change = change.cache_change();
            let timestamp = cache_change.timestamp();

            if cache_change.data_value().len() > data_max_size_serialized {
                let data_frag_submessage_list =
                    cache_change.as_data_frag_submessages(data_max_size_serialized, reader_id);
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
                    cache_change.as_data_submessage(reader_id),
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

#[allow(clippy::too_many_arguments)]
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
                let cache_change = change.cache_change();
                if cache_change.data_value().len() > data_max_size_serialized {
                    directly_send_data_frag(
                        reader_proxy,
                        cache_change,
                        writer_id,
                        data_max_size_serialized,
                        header,
                        first_sn,
                        last_sn,
                        transport,
                    );
                    return;
                } else {
                    submessages.push(info_timestamp_submessage(cache_change.timestamp()));
                    submessages.push(RtpsSubmessageKind::Data(
                        cache_change.as_data_submessage(reader_id),
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
                let cache_change = change_for_reader.cache_change();
                if cache_change.data_value().len() > data_max_size_serialized {
                    directly_send_data_frag(
                        reader_proxy,
                        cache_change,
                        writer_id,
                        data_max_size_serialized,
                        header,
                        first_sn,
                        last_sn,
                        transport,
                    );
                    return;
                } else {
                    submessages.push(info_timestamp_submessage(cache_change.timestamp()));
                    submessages.push(RtpsSubmessageKind::Data(
                        cache_change.as_data_submessage(reader_id),
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

#[allow(clippy::too_many_arguments)]
fn directly_send_data_frag(
    reader_proxy: &mut WriterAssociatedReaderProxy,
    cache_change: &RtpsWriterCacheChange,
    writer_id: EntityId,
    data_max_size_serialized: usize,
    header: RtpsMessageHeader,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    transport: &mut impl TransportWrite,
) {
    let reader_id = reader_proxy.remote_reader_guid().entity_id();
    let timestamp = cache_change.timestamp();

    let mut data_frag_submessage_list = cache_change
        .as_data_frag_submessages(data_max_size_serialized, reader_id)
        .into_iter()
        .peekable();

    while let Some(data_frag_submessage) = data_frag_submessage_list.next() {
        let writer_sn = data_frag_submessage.writer_sn;
        let last_fragment_num = FragmentNumber::new(
            u32::from(data_frag_submessage.fragment_starting_num)
                + u16::from(data_frag_submessage.fragments_in_submessage) as u32
                - 1,
        );

        let info_dst = info_destination_submessage(reader_proxy.remote_reader_guid().prefix());
        let into_timestamp = info_timestamp_submessage(timestamp);
        let data_frag = RtpsSubmessageKind::DataFrag(data_frag_submessage);

        let is_last_fragment = data_frag_submessage_list.peek().is_none();
        let submessages = if is_last_fragment {
            let heartbeat_frag = reader_proxy.heartbeat_frag_machine().submessage(
                writer_id,
                writer_sn,
                last_fragment_num,
            );
            vec![info_dst, into_timestamp, data_frag, heartbeat_frag]
        } else {
            let heartbeat = reader_proxy
                .heartbeat_machine()
                .submessage(writer_id, first_sn, last_sn);
            vec![info_dst, into_timestamp, data_frag, heartbeat]
        };
        transport.write(
            &RtpsMessage::new(header, submessages),
            reader_proxy.unicast_locator_list(),
        )
    }
}

fn builtin_stateful_writer_send_message(
    writer: &BuiltinStatefulWriter,
    header: RtpsMessageHeader,
    transport: &mut impl TransportWrite,
) {
    let data_max_size_serialized = writer.data_max_size_serialized();
    let writer_id = writer.guid().entity_id();
    let first_sn = writer
        .change_list()
        .into_iter()
        .map(|x| x.sequence_number())
        .min()
        .unwrap_or_else(|| SequenceNumber::new(1));
    let last_sn = writer
        .change_list()
        .into_iter()
        .map(|x| x.sequence_number())
        .max()
        .unwrap_or_else(|| SequenceNumber::new(0));
    let heartbeat_period = writer.heartbeat_period();
    for mut reader_proxy in &mut writer.matched_reader_list() {
        send_message_reliable_reader_proxy(
            &mut reader_proxy,
            data_max_size_serialized,
            header,
            transport,
            writer_id,
            first_sn,
            last_sn,
            heartbeat_period,
        )
    }
}

fn discover_matched_writers(domain_participant: &DomainParticipantImpl) -> DdsResult<()> {
    let samples = domain_participant
        .get_builtin_subscriber()
        .sedp_builtin_publications_reader()
        .read::<DiscoveredWriterData>(
            i32::MAX,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
            None,
        )?;

    for discovered_writer_data_sample in samples.into_iter() {
        match discovered_writer_data_sample.sample_info.instance_state {
            InstanceStateKind::Alive => {
                if let Some(discovered_writer_data) = discovered_writer_data_sample.data {
                    if !domain_participant.is_publication_ignored(
                        discovered_writer_data
                            .writer_proxy
                            .remote_writer_guid
                            .into(),
                    ) {
                        let remote_writer_guid_prefix = discovered_writer_data
                            .writer_proxy
                            .remote_writer_guid
                            .prefix();
                        let writer_parent_participant_guid =
                            Guid::new(remote_writer_guid_prefix, ENTITYID_PARTICIPANT);

                        if let Some((_, discovered_participant_data)) = domain_participant
                            .discovered_participant_list()
                            .into_iter()
                            .find(|&(h, _)| {
                                h == &InstanceHandle::from(writer_parent_participant_guid)
                            })
                        {
                            for subscriber in &domain_participant.user_defined_subscriber_list() {
                                subscriber_add_matched_writer(
                                    subscriber,
                                    &discovered_writer_data,
                                    discovered_participant_data.default_unicast_locator_list(),
                                    discovered_participant_data.default_multicast_locator_list(),
                                    &mut domain_participant.get_status_listener_lock(),
                                );
                            }
                        }
                    }
                }
            }
            InstanceStateKind::NotAliveDisposed => {
                for subscriber in &domain_participant.user_defined_subscriber_list() {
                    for data_reader in &subscriber.data_reader_list() {
                        data_reader.remove_matched_writer(
                            discovered_writer_data_sample.sample_info.instance_handle,
                            &mut subscriber.get_status_listener_lock(),
                            &mut domain_participant.get_status_listener_lock(),
                        )
                    }
                }
            }
            InstanceStateKind::NotAliveNoWriters => todo!(),
        }
    }

    Ok(())
}

pub fn subscriber_add_matched_writer(
    user_defined_subscriber: &UserDefinedSubscriber,
    discovered_writer_data: &DiscoveredWriterData,
    default_unicast_locator_list: &[Locator],
    default_multicast_locator_list: &[Locator],
    participant_status_listener: &mut StatusListener<dyn DomainParticipantListener + Send + Sync>,
) {
    let is_discovered_writer_regex_matched_to_subscriber = if let Ok(d) = glob_to_regex(
        &discovered_writer_data
            .publication_builtin_topic_data
            .partition
            .name,
    ) {
        d.is_match(&user_defined_subscriber.get_qos().partition.name)
    } else {
        false
    };

    let is_subscriber_regex_matched_to_discovered_writer =
        if let Ok(d) = glob_to_regex(&user_defined_subscriber.get_qos().partition.name) {
            d.is_match(
                &discovered_writer_data
                    .publication_builtin_topic_data
                    .partition
                    .name,
            )
        } else {
            false
        };

    let is_partition_string_matched = discovered_writer_data
        .publication_builtin_topic_data
        .partition
        .name
        == user_defined_subscriber.get_qos().partition.name;

    if is_discovered_writer_regex_matched_to_subscriber
        || is_subscriber_regex_matched_to_discovered_writer
        || is_partition_string_matched
    {
        for data_reader in &user_defined_subscriber.data_reader_list() {
            data_reader.add_matched_writer(
                discovered_writer_data,
                default_unicast_locator_list,
                default_multicast_locator_list,
                &mut user_defined_subscriber.get_status_listener_lock(),
                participant_status_listener,
                &user_defined_subscriber.get_qos(),
            )
        }
    }
}
