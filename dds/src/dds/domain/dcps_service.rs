use std::{
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    sync::{
        atomic::{self, AtomicBool},
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread::JoinHandle,
};

use fnmatch_regex::glob_to_regex;

use crate::{
    domain::domain_participant_factory::THE_DDS_DOMAIN_PARTICIPANT_FACTORY,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy, DCPS_SUBSCRIPTION},
            discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
            discovered_writer_data::{DiscoveredWriterData, WriterProxy, DCPS_PUBLICATION},
            spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
        },
        dds::{
            dds_data_reader::DdsDataReader,
            dds_data_writer::DdsDataWriter,
            dds_domain_participant::{
                AnnounceKind, DdsDomainParticipant, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            },
            dds_subscriber::DdsSubscriber,
            nodes::{
                DataReaderNode, DataWriterNode, SubscriberNode, SubscriberNodeKind, TopicNode,
            },
            participant_discovery::ParticipantDiscovery,
            status_listener::ListenerTriggerKind,
        },
        rtps::{
            discovery_types::BuiltinEndpointSet,
            history_cache::RtpsWriterCacheChange,
            messages::{
                overall_structure::RtpsMessageHeader,
                submessage_elements::SequenceNumberSet,
                submessages::{GapSubmessage, InfoDestinationSubmessage, InfoTimestampSubmessage},
                types::{FragmentNumber, ProtocolId},
                RtpsMessage, RtpsSubmessageKind,
            },
            reader_locator::WriterAssociatedReaderLocator,
            reader_proxy::{RtpsReaderProxy, WriterAssociatedReaderProxy},
            stateful_reader::RtpsStatefulReader,
            stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter,
            transport::TransportWrite,
            types::{
                DurabilityKind, EntityId, Guid, GuidPrefix, Locator, ReliabilityKind,
                SequenceNumber, ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN,
            },
            writer_proxy::RtpsWriterProxy,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::{condvar::DdsCondvar, shared_object::DdsRwLock},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        status::{
            InconsistentTopicStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
            SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind, Time},
    },
    subscription::{
        sample_info::{
            InstanceStateKind, SampleStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
        },
        subscriber::Subscriber,
    },
    topic_definition::type_support::{DdsSerialize, DdsSerializedKey, DdsType},
};

pub struct DcpsService {
    quit: Arc<AtomicBool>,
    threads: DdsRwLock<Vec<JoinHandle<()>>>,
    sedp_condvar: DdsCondvar,
    user_defined_data_send_condvar: DdsCondvar,
    sender_socket: UdpSocket,
    announce_sender: SyncSender<AnnounceKind>,
    default_unicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
}

impl Drop for DcpsService {
    fn drop(&mut self) {
        self.quit.store(true, atomic::Ordering::SeqCst);

        self.sedp_condvar.notify_all();
        self.user_defined_data_send_condvar.notify_all();
        self.announce_sender
            .send(AnnounceKind::DeletedParticipant)
            .ok();

        // Send shutdown messages
        if let Some(default_unicast_locator) = self.default_unicast_locator_list.get(0) {
            let port: u32 = default_unicast_locator.port().into();
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port as u16);
            self.sender_socket.send_to(&[0], addr).ok();
        }

        if let Some(metatraffic_unicast_locator) = self.metatraffic_unicast_locator_list.get(0) {
            let port: u32 = metatraffic_unicast_locator.port().into();
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port as u16);
            self.sender_socket.send_to(&[0], addr).ok();
        }

        if let Some(metatraffic_multicast_transport) =
            self.metatraffic_multicast_locator_list.get(0)
        {
            let addr: [u8; 16] = metatraffic_multicast_transport.address().into();
            let port: u32 = metatraffic_multicast_transport.port().into();
            let addr = SocketAddrV4::new(
                Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                port as u16,
            );
            self.sender_socket.send_to(&[0], addr).ok();
        }

        while let Some(thread) = self.threads.write_lock().pop() {
            thread.join().unwrap();
        }
    }
}

impl DcpsService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        participant_guid_prefix: GuidPrefix,
        default_unicast_locator_list: Vec<Locator>,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        sedp_condvar: &DdsCondvar,
        user_defined_data_send_condvar: &DdsCondvar,
        announce_sender: SyncSender<AnnounceKind>,
        announce_receiver: Receiver<AnnounceKind>,
    ) -> DdsResult<Self> {
        let quit = Arc::new(AtomicBool::new(false));
        let mut threads = Vec::new();

        let (_listener_sender, listener_receiver) = mpsc::channel();

        //  ////////////// Entity announcer thread
        {
            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Ok(announce_kind) = announce_receiver.recv() {
                    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
                        &participant_guid_prefix,
                        |dp| {
                            if let Some(dp) = dp {
                                announce_entity(dp, announce_kind);
                            }
                        },
                    )
                }
            }));
        }

        // //////////// Unicast metatraffic Communication send
        {
            let socket = UdpSocket::bind("0.0.0.0:0000").unwrap();

            let mut metatraffic_unicast_transport_send = UdpTransportWrite::new(socket);
            let task_quit = quit.clone();
            let sedp_condvar_clone = sedp_condvar.clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }
                let _r = sedp_condvar_clone.wait_timeout(Duration::new(0, 500000000));
                THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
                    &participant_guid_prefix,
                    |dp| {
                        if let Some(dp) = dp {
                            send_user_defined_message(dp, &mut metatraffic_unicast_transport_send);
                        }
                    },
                );
            }));
        }

        // //////////// User-defined Communication send
        {
            let socket = UdpSocket::bind("0.0.0.0:0000").unwrap();
            let mut default_unicast_transport_send = UdpTransportWrite::new(socket);
            let task_quit = quit.clone();
            let user_defined_data_send_condvar_clone = user_defined_data_send_condvar.clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                let _r = user_defined_data_send_condvar_clone
                    .wait_timeout(Duration::new(0, 100_000_000));

                THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
                    &participant_guid_prefix,
                    |dp| {
                        if let Some(dp) = dp {
                            user_defined_communication_send(
                                dp,
                                &mut default_unicast_transport_send,
                            );
                        }
                    },
                )
            }));
        }

        //////////// Listener thread
        {
            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Ok(l) = listener_receiver.recv() {
                    match l {
                        ListenerTriggerKind::RequestedDeadlineMissed(dr) => {
                            on_requested_deadline_missed_communication_change(dr)
                        }
                        ListenerTriggerKind::OnDataAvailable(dr) => {
                            on_data_available_communication_change(dr)
                        }
                        ListenerTriggerKind::SubscriptionMatched(dr) => {
                            on_subscription_matched_communication_change(dr)
                        }
                        ListenerTriggerKind::RequestedIncompatibleQos(dr) => {
                            on_requested_incompatible_qos_communication_change(dr)
                        }
                        ListenerTriggerKind::OnSampleRejected(dr) => {
                            on_sample_rejected_communication_change(dr)
                        }
                        ListenerTriggerKind::OnSampleLost(dr) => {
                            on_sample_lost_communication_change(dr)
                        }
                        ListenerTriggerKind::OfferedIncompatibleQos(dw) => {
                            on_offered_incompatible_qos_communication_change(dw)
                        }
                        ListenerTriggerKind::PublicationMatched(dw) => {
                            on_publication_matched_communication_change(dw)
                        }
                        ListenerTriggerKind::InconsistentTopic(t) => {
                            on_inconsistent_topic_communication_change(t)
                        }
                    }
                }
            }));
        }

        let sender_socket = UdpSocket::bind("0.0.0.0:0000").unwrap();

        let sedp_condvar = sedp_condvar.clone();
        let user_defined_data_send_condvar = user_defined_data_send_condvar.clone();
        Ok(DcpsService {
            default_unicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            quit,
            threads: DdsRwLock::new(threads),
            sedp_condvar,
            user_defined_data_send_condvar,
            sender_socket,
            announce_sender,
        })
    }

    pub fn _sedp_condvar(&self) -> &DdsCondvar {
        &self.sedp_condvar
    }

    pub fn user_defined_data_send_condvar(&self) -> &DdsCondvar {
        &self.user_defined_data_send_condvar
    }

    pub fn announce_sender(&self) -> &SyncSender<AnnounceKind> {
        &self.announce_sender
    }
}

fn on_requested_deadline_missed_communication_change(data_reader_node: DataReaderNode) {
    fn get_requested_deadline_missed_status(
        data_reader_node: &DataReaderNode,
    ) -> DdsResult<RequestedDeadlineMissedStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .get_participant_mut(&data_reader_node.parent_participant().prefix(), |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_requested_deadline_missed_status(dp.ok_or(DdsError::AlreadyDeleted)?, data_reader_node.guid(), data_reader_node.parent_subscriber())
            })
    }

    let status_kind = StatusKind::RequestedDeadlineMissed;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_requested_deadline_missed_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_requested_deadline_missed(data_reader_node, status);
                }
                true
            }
            _ => false,
        },
    );

    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_requested_deadline_missed_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_requested_deadline_missed(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );

        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_requested_deadline_missed_status(&data_reader_node)
                        {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_requested_deadline_missed(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_data_available_communication_change(data_reader_node: DataReaderNode) {
    let data_on_reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
        &data_reader_node.parent_subscriber(),
        |subscriber_listener| match subscriber_listener {
            Some(l) if l.is_enabled(&StatusKind::DataOnReaders) => {
                l.listener_mut()
                    .as_mut()
                    .expect("Listener should be some")
                    .on_data_on_readers(&Subscriber::new(SubscriberNodeKind::Listener(
                        SubscriberNode::new(
                            data_reader_node.parent_subscriber(),
                            data_reader_node.parent_participant(),
                        ),
                    )));
                true
            }
            _ => false,
        },
    );
    if !data_on_reader_listener {
        let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
            &data_reader_node.guid(),
            |data_reader_listener| match data_reader_listener {
                Some(l) if l.is_enabled(&StatusKind::DataAvailable) => {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_data_available(data_reader_node);
                    true
                }
                _ => false,
            },
        );
        if !reader_listener {
            let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
                &data_reader_node.parent_subscriber(),
                |subscriber_listener| match subscriber_listener {
                    Some(l) if l.is_enabled(&StatusKind::DataAvailable) => {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_data_available(&data_reader_node);

                        true
                    }
                    _ => false,
                },
            );
            if !subscriber_listener {
                THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                    &data_reader_node.parent_participant(),
                    |participant_listener| match participant_listener {
                        Some(l) if l.is_enabled(&StatusKind::DataAvailable) => l
                            .listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_data_available(&data_reader_node),
                        _ => (),
                    },
                );
            }
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
        &data_reader_node.parent_subscriber(),
        |subscriber_listener| {
            if let Some(l) = subscriber_listener {
                l.add_communication_state(StatusKind::DataOnReaders);
            }
        },
    );

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(StatusKind::DataAvailable);
            }
        },
    )
}

fn on_subscription_matched_communication_change(data_reader_node: DataReaderNode) {
    fn get_subscription_matched_status(
        data_reader_node: &DataReaderNode,
    ) -> DdsResult<SubscriptionMatchedStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_reader_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_subscription_matched_status(dp.ok_or(DdsError::AlreadyDeleted)?, data_reader_node.guid(), data_reader_node.parent_subscriber())
            },
        )
    }

    let status_kind = StatusKind::SubscriptionMatched;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_subscription_matched_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_subscription_matched(data_reader_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_subscription_matched_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_subscription_matched(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_subscription_matched_status(&data_reader_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_subscription_matched(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_requested_incompatible_qos_communication_change(data_reader_node: DataReaderNode) {
    fn get_requested_incompatible_qos_status(
        data_reader_node: &DataReaderNode,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_reader_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_requested_incompatible_qos_status(dp.ok_or(DdsError::AlreadyDeleted)?, data_reader_node.guid(), data_reader_node.parent_subscriber())
            },
        )
    }

    let status_kind = StatusKind::RequestedIncompatibleQos;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_requested_incompatible_qos_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_requested_incompatible_qos(data_reader_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_requested_incompatible_qos_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_requested_incompatible_qos(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_requested_incompatible_qos_status(&data_reader_node)
                        {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_requested_incompatible_qos(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_sample_rejected_communication_change(data_reader_node: DataReaderNode) {
    fn get_sample_rejected_status(
        data_reader_node: &DataReaderNode,
    ) -> DdsResult<SampleRejectedStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .get_participant_mut(&data_reader_node.parent_participant().prefix(), |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_sample_rejected_status(dp.ok_or(DdsError::AlreadyDeleted)?, data_reader_node.guid(), data_reader_node.parent_subscriber())
            })
    }

    let status_kind = StatusKind::SampleRejected;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_sample_rejected_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_sample_rejected(data_reader_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_sample_rejected_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_sample_rejected(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_sample_rejected_status(&data_reader_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_sample_rejected(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_sample_lost_communication_change(data_reader_node: DataReaderNode) {
    fn get_sample_lost_status(data_reader_node: &DataReaderNode) -> DdsResult<SampleLostStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_reader_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_reader::get_sample_lost_status(
                    dp.ok_or(DdsError::AlreadyDeleted)?,
                    data_reader_node.guid(),
                    data_reader_node.parent_subscriber(),
                )
            },
        )
    }

    let status_kind = StatusKind::SampleLost;
    let reader_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| match data_reader_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_sample_lost_status(&data_reader_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_sample_lost(data_reader_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !reader_listener {
        let subscriber_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_subscriber_listener(
            &data_reader_node.parent_subscriber(),
            |subscriber_listener| match subscriber_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_sample_lost_status(&data_reader_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_sample_lost(&data_reader_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !subscriber_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_reader_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_sample_lost_status(&data_reader_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_sample_lost(&data_reader_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_reader_listener(
        &data_reader_node.guid(),
        |data_reader_listener| {
            if let Some(l) = data_reader_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_offered_incompatible_qos_communication_change(data_writer_node: DataWriterNode) {
    fn get_offered_incompatible_qos_status(
        data_writer_node: &DataWriterNode,
    ) -> DdsResult<OfferedIncompatibleQosStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_writer_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_writer::get_offered_incompatible_qos_status(
                    dp.ok_or(DdsError::AlreadyDeleted)?,
                    data_writer_node.guid(),
                    data_writer_node.parent_publisher(),
                )
            },
        )
    }

    let status_kind = StatusKind::OfferedIncompatibleQos;
    let writer_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_writer_listener(
        &data_writer_node.guid(),
        |data_writer_listener| match data_writer_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_offered_incompatible_qos_status(&data_writer_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_offered_incompatible_qos(data_writer_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !writer_listener {
        let publisher_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_publisher_listener(
            &data_writer_node.parent_publisher(),
            |publisher_listener| match publisher_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_offered_incompatible_qos_status(&data_writer_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_offered_incompatible_qos(&data_writer_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !publisher_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_writer_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_offered_incompatible_qos_status(&data_writer_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_offered_incompatible_qos(&data_writer_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_writer_listener(
        &data_writer_node.guid(),
        |data_writer_listener| {
            if let Some(l) = data_writer_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_publication_matched_communication_change(data_writer_node: DataWriterNode) {
    fn get_publication_matched_status(
        data_writer_node: &DataWriterNode,
    ) -> DdsResult<PublicationMatchedStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &data_writer_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_data_writer::get_publication_matched_status(dp.ok_or(DdsError::AlreadyDeleted)?,data_writer_node.guid(),
                data_writer_node.parent_publisher(),)
            },
        )
    }

    let status_kind = StatusKind::PublicationMatched;
    let writer_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_writer_listener(
        &data_writer_node.guid(),
        |data_writer_listener| match data_writer_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_publication_matched_status(&data_writer_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_publication_matched(data_writer_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !writer_listener {
        let publisher_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_publisher_listener(
            &data_writer_node.parent_publisher(),
            |publisher_listener| match publisher_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_publication_matched_status(&data_writer_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_publication_matched(&data_writer_node, status)
                    }
                    true
                }
                _ => false,
            },
        );
        if !publisher_listener {
            THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
                &data_writer_node.parent_participant(),
                |participant_listener| match participant_listener {
                    Some(l) if l.is_enabled(&status_kind) => {
                        if let Ok(status) = get_publication_matched_status(&data_writer_node) {
                            l.listener_mut()
                                .as_mut()
                                .expect("Listener should be some")
                                .on_publication_matched(&data_writer_node, status)
                        }
                    }
                    _ => (),
                },
            );
        }
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_data_writer_listener(
        &data_writer_node.guid(),
        |data_writer_listener| {
            if let Some(l) = data_writer_listener {
                l.add_communication_state(status_kind);
            }
        },
    )
}

fn on_inconsistent_topic_communication_change(topic_node: TopicNode) {
    fn get_inconsistent_topic_status(topic_node: &TopicNode) -> DdsResult<InconsistentTopicStatus> {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(
            &topic_node.parent_participant().prefix(),
            |dp| {
                crate::implementation::behavior::user_defined_topic::get_inconsistent_topic_status(
                    dp.ok_or(DdsError::AlreadyDeleted)?,
                    topic_node.guid(),
                )
            },
        )
    }

    let status_kind = StatusKind::InconsistentTopic;
    let topic_listener = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_topic_listener(
        &topic_node.guid(),
        |topic_listener| match topic_listener {
            Some(l) if l.is_enabled(&status_kind) => {
                if let Ok(status) = get_inconsistent_topic_status(&topic_node) {
                    l.listener_mut()
                        .as_mut()
                        .expect("Listener should be some")
                        .trigger_on_inconsistent_topic(topic_node, status)
                }
                true
            }
            _ => false,
        },
    );
    if !topic_listener {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_domain_participant_listener(
            &topic_node.parent_participant(),
            |participant_listener| match participant_listener {
                Some(l) if l.is_enabled(&status_kind) => {
                    if let Ok(status) = get_inconsistent_topic_status(&topic_node) {
                        l.listener_mut()
                            .as_mut()
                            .expect("Listener should be some")
                            .on_inconsistent_topic(&topic_node, status)
                    }
                }
                _ => (),
            },
        );
    }

    THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_topic_listener(&topic_node.guid(), |topic_listener| {
        if let Some(l) = topic_listener {
            l.add_communication_state(status_kind);
        }
    })
}

fn announce_created_data_reader(
    domain_participant: &mut DdsDomainParticipant,
    discovered_reader_data: DiscoveredReaderData,
) {
    let reader_proxy = ReaderProxy::new(
        discovered_reader_data.reader_proxy().remote_reader_guid(),
        discovered_reader_data
            .reader_proxy()
            .remote_group_entity_id(),
        domain_participant.default_unicast_locator_list().to_vec(),
        domain_participant.default_multicast_locator_list().to_vec(),
        discovered_reader_data.reader_proxy().expects_inline_qos(),
    );
    let reader_data = &DiscoveredReaderData::new(
        reader_proxy,
        discovered_reader_data
            .subscription_builtin_topic_data()
            .clone(),
    );

    let mut serialized_data = Vec::new();
    reader_data
        .dds_serialize(&mut serialized_data)
        .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();
    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredReaderData::type_name())
        .unwrap()
        .write_w_timestamp(
            serialized_data,
            reader_data.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_created_data_writer(
    domain_participant: &mut DdsDomainParticipant,
    discovered_writer_data: DiscoveredWriterData,
) {
    let writer_data = &DiscoveredWriterData::new(
        discovered_writer_data.dds_publication_data().clone(),
        WriterProxy::new(
            discovered_writer_data.writer_proxy().remote_writer_guid(),
            discovered_writer_data
                .writer_proxy()
                .remote_group_entity_id(),
            domain_participant.default_unicast_locator_list().to_vec(),
            domain_participant.default_multicast_locator_list().to_vec(),
            discovered_writer_data
                .writer_proxy()
                .data_max_size_serialized(),
        ),
    );

    let mut serialized_data = Vec::new();
    writer_data
        .dds_serialize(&mut serialized_data)
        .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredWriterData::type_name())
        .unwrap()
        .write_w_timestamp(
            serialized_data,
            writer_data.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_created_topic(
    domain_participant: &mut DdsDomainParticipant,
    discovered_topic: DiscoveredTopicData,
) {
    let mut serialized_data = Vec::new();
    discovered_topic
        .dds_serialize(&mut serialized_data)
        .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredTopicData::type_name())
        .unwrap()
        .write_w_timestamp(
            serialized_data,
            discovered_topic.get_serialized_key(),
            None,
            timestamp,
        )
        .expect("Should not fail to write built-in message");
}

fn announce_deleted_reader(
    domain_participant: &mut DdsDomainParticipant,
    reader_handle: InstanceHandle,
) {
    let serialized_key = DdsSerializedKey::from(reader_handle.as_ref());
    let instance_serialized_key =
        cdr::serialize::<_, _, cdr::CdrLe>(&serialized_key, cdr::Infinite)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
            .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredReaderData::type_name())
        .unwrap()
        .dispose_w_timestamp(instance_serialized_key, reader_handle, timestamp)
        .expect("Should not fail to write built-in message");
}

fn announce_deleted_writer(
    domain_participant: &mut DdsDomainParticipant,
    writer_handle: InstanceHandle,
) {
    let serialized_key = DdsSerializedKey::from(writer_handle.as_ref());
    let instance_serialized_key =
        cdr::serialize::<_, _, cdr::CdrLe>(&serialized_key, cdr::Infinite)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
            .expect("Failed to serialize data");

    let timestamp = domain_participant.get_current_time();

    domain_participant
        .get_builtin_publisher_mut()
        .stateful_data_writer_list_mut()
        .iter_mut()
        .find(|x| x.get_type_name() == DiscoveredWriterData::type_name())
        .unwrap()
        .dispose_w_timestamp(instance_serialized_key, writer_handle, timestamp)
        .expect("Should not fail to write built-in message");
}

fn remove_stale_writer_changes(writer: &mut DdsDataWriter<RtpsStatefulWriter>, now: Time) {
    let timespan_duration = writer.get_qos().lifespan.duration;
    writer.remove_change(|cc| DurationKind::Finite(now - cc.timestamp()) > timespan_duration);
}

fn send_message_best_effort_reader_locator(
    reader_locator: &mut WriterAssociatedReaderLocator,
    header: RtpsMessageHeader,
    transport: &mut impl TransportWrite,
    writer_id: EntityId,
) {
    let mut submessages = Vec::new();
    while let Some(change) = reader_locator.next_unsent_change() {
        // The post-condition:
        // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
        // should be full-filled by next_unsent_change()
        if let Some(cache_change) = change.cache_change() {
            let info_ts_submessage = info_timestamp_submessage(cache_change.timestamp());
            let data_submessage = cache_change.as_data_submessage(ENTITYID_UNKNOWN);
            submessages.push(info_ts_submessage);
            submessages.push(RtpsSubmessageKind::Data(data_submessage));
        } else {
            let gap_submessage = gap_submessage(writer_id, change.sequence_number());
            submessages.push(gap_submessage);
        }
    }
    if !submessages.is_empty() {
        transport.write(
            &RtpsMessage::new(header, submessages),
            &[reader_locator.locator()],
        )
    }
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

fn gap_submessage<'a>(
    writer_id: EntityId,
    gap_sequence_number: SequenceNumber,
) -> RtpsSubmessageKind<'a> {
    RtpsSubmessageKind::Gap(GapSubmessage {
        endianness_flag: true,
        reader_id: ENTITYID_UNKNOWN,
        writer_id,
        gap_start: gap_sequence_number,
        gap_list: SequenceNumberSet {
            base: gap_sequence_number,
            set: vec![],
        },
    })
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

fn stateless_writer_send_message(
    writer: &mut DdsDataWriter<RtpsStatelessWriter>,
    header: RtpsMessageHeader,
    transport: &mut impl TransportWrite,
) {
    let writer_id = writer.guid().entity_id();
    for mut rl in &mut writer.reader_locator_list().into_iter() {
        send_message_best_effort_reader_locator(&mut rl, header, transport, writer_id);
    }
}

fn user_defined_stateful_writer_send_message(
    writer: &mut DdsDataWriter<RtpsStatefulWriter>,
    header: RtpsMessageHeader,
    transport: &mut impl TransportWrite,
) {
    let data_max_size_serialized = writer.data_max_size_serialized();
    let writer_id = writer.guid().entity_id();
    let first_sn = writer
        .change_list()
        .iter()
        .map(|x| x.sequence_number())
        .min()
        .unwrap_or_else(|| SequenceNumber::new(1));
    let last_sn = writer
        .change_list()
        .iter()
        .map(|x| x.sequence_number())
        .max()
        .unwrap_or_else(|| SequenceNumber::new(0));
    let heartbeat_period = writer.heartbeat_period();
    for reader_proxy in &mut writer.matched_reader_list() {
        match reader_proxy.reliability() {
            ReliabilityKind::BestEffort => send_message_best_effort_reader_proxy(
                reader_proxy,
                data_max_size_serialized,
                header,
                transport,
            ),
            ReliabilityKind::Reliable => send_message_reliable_reader_proxy(
                reader_proxy,
                data_max_size_serialized,
                header,
                transport,
                writer_id,
                first_sn,
                last_sn,
                heartbeat_period,
            ),
        }
    }
}

fn discover_matched_writers(
    domain_participant: &mut DdsDomainParticipant,
    listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
) -> DdsResult<()> {
    let samples = domain_participant
        .get_builtin_subscriber_mut()
        .stateful_data_reader_list_mut()
        .iter_mut()
        .find(|x| x.get_topic_name() == DCPS_PUBLICATION)
        .unwrap()
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
                            .writer_proxy()
                            .remote_writer_guid()
                            .into(),
                    ) {
                        let remote_writer_guid_prefix = discovered_writer_data
                            .writer_proxy()
                            .remote_writer_guid()
                            .prefix();
                        let writer_parent_participant_guid =
                            Guid::new(remote_writer_guid_prefix, ENTITYID_PARTICIPANT);

                        if let Some((
                            default_unicast_locator_list,
                            default_multicast_locator_list,
                        )) = domain_participant
                            .discovered_participant_list()
                            .into_iter()
                            .find(|&(h, _)| {
                                h == &InstanceHandle::from(writer_parent_participant_guid)
                            })
                            .map(|(_, discovered_participant_data)| {
                                (
                                    discovered_participant_data
                                        .participant_proxy()
                                        .default_unicast_locator_list()
                                        .to_vec(),
                                    discovered_participant_data
                                        .participant_proxy()
                                        .default_multicast_locator_list()
                                        .to_vec(),
                                )
                            })
                        {
                            let domain_participant_guid = domain_participant.guid();
                            for subscriber in domain_participant.user_defined_subscriber_list_mut()
                            {
                                subscriber_add_matched_writer(
                                    subscriber,
                                    &discovered_writer_data,
                                    &default_unicast_locator_list,
                                    &default_multicast_locator_list,
                                    domain_participant_guid,
                                    listener_sender,
                                );
                            }
                        }
                    }
                }
            }
            InstanceStateKind::NotAliveDisposed => {
                let domain_participant_guid = domain_participant.guid();
                for subscriber in domain_participant.user_defined_subscriber_list_mut() {
                    let subscriber_guid = subscriber.guid();
                    for data_reader in subscriber.stateful_data_reader_list_mut() {
                        data_reader.remove_matched_writer(
                            discovered_writer_data_sample.sample_info.instance_handle,
                            domain_participant_guid,
                            subscriber_guid,
                            listener_sender,
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
    user_defined_subscriber: &mut DdsSubscriber,
    discovered_writer_data: &DiscoveredWriterData,
    default_unicast_locator_list: &[Locator],
    default_multicast_locator_list: &[Locator],
    parent_participant_guid: Guid,
    listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
) {
    let is_discovered_writer_regex_matched_to_subscriber = if let Ok(d) = glob_to_regex(
        &discovered_writer_data
            .clone()
            .dds_publication_data()
            .partition()
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
                    .clone()
                    .dds_publication_data()
                    .partition()
                    .name,
            )
        } else {
            false
        };

    let is_partition_string_matched = discovered_writer_data
        .clone()
        .dds_publication_data()
        .partition()
        .name
        == user_defined_subscriber.get_qos().partition.name;

    if is_discovered_writer_regex_matched_to_subscriber
        || is_subscriber_regex_matched_to_discovered_writer
        || is_partition_string_matched
    {
        let user_defined_subscriber_qos = user_defined_subscriber.get_qos();
        let user_defined_subscriber_guid = user_defined_subscriber.guid();
        for data_reader in user_defined_subscriber.stateful_data_reader_list_mut() {
            data_reader.add_matched_writer(
                discovered_writer_data,
                default_unicast_locator_list,
                default_multicast_locator_list,
                &user_defined_subscriber_qos,
                user_defined_subscriber_guid,
                parent_participant_guid,
                listener_sender,
            )
        }
    }
}

fn discover_matched_participants(
    domain_participant: &mut DdsDomainParticipant,
    sedp_condvar: &DdsCondvar,
) -> DdsResult<()> {
    while let Ok(samples) = domain_participant
        .get_builtin_subscriber_mut()
        .stateless_data_reader_list_mut()
        .iter_mut()
        .find(|x| x.get_topic_name() == DCPS_PARTICIPANT)
        .unwrap()
        .read(
            1,
            &[SampleStateKind::NotRead],
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
            None,
        )
    {
        for discovered_participant_data_sample in samples.into_iter() {
            if let Some(discovered_participant_data) = discovered_participant_data_sample.data {
                add_discovered_participant(domain_participant, discovered_participant_data);
                sedp_condvar.notify_all();
            }
        }
    }

    Ok(())
}

fn add_discovered_participant(
    domain_participant: &mut DdsDomainParticipant,
    discovered_participant_data: SpdpDiscoveredParticipantData,
) {
    if ParticipantDiscovery::new(
        &discovered_participant_data,
        domain_participant.get_domain_id(),
        domain_participant.get_domain_tag(),
    )
    .is_ok()
        && !domain_participant
            .is_participant_ignored(discovered_participant_data.get_serialized_key().into())
    {
        add_matched_publications_detector(
            domain_participant
                .get_builtin_publisher_mut()
                .stateful_data_writer_list_mut()
                .iter_mut()
                .find(|x| x.get_topic_name() == DCPS_PUBLICATION)
                .unwrap(),
            &discovered_participant_data,
        );

        add_matched_publications_announcer(
            domain_participant
                .get_builtin_subscriber_mut()
                .stateful_data_reader_list_mut()
                .iter_mut()
                .find(|x| x.get_topic_name() == DCPS_PUBLICATION)
                .unwrap(),
            &discovered_participant_data,
        );

        add_matched_subscriptions_detector(
            domain_participant
                .get_builtin_publisher_mut()
                .stateful_data_writer_list_mut()
                .iter_mut()
                .find(|x| x.get_topic_name() == DCPS_SUBSCRIPTION)
                .unwrap(),
            &discovered_participant_data,
        );

        add_matched_subscriptions_announcer(
            domain_participant
                .get_builtin_subscriber_mut()
                .stateful_data_reader_list_mut()
                .iter_mut()
                .find(|x| x.get_topic_name() == DCPS_SUBSCRIPTION)
                .unwrap(),
            &discovered_participant_data,
        );

        add_matched_topics_detector(
            domain_participant
                .get_builtin_publisher_mut()
                .stateful_data_writer_list_mut()
                .iter_mut()
                .find(|x| x.get_topic_name() == DCPS_TOPIC)
                .unwrap(),
            &discovered_participant_data,
        );

        add_matched_topics_announcer(
            domain_participant
                .get_builtin_subscriber_mut()
                .stateful_data_reader_list_mut()
                .iter_mut()
                .find(|x| x.get_topic_name() == DCPS_TOPIC)
                .unwrap(),
            &discovered_participant_data,
        );

        domain_participant.discovered_participant_add(
            discovered_participant_data.get_serialized_key().into(),
            discovered_participant_data,
        );
    }
}

fn add_matched_subscriptions_announcer(
    reader: &mut DdsDataReader<RtpsStatefulReader>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
    {
        let remote_writer_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let data_max_size_serialized = None;

        let proxy = RtpsWriterProxy::new(
            remote_writer_guid,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            data_max_size_serialized,
            remote_group_entity_id,
        );
        reader.matched_writer_add(proxy);
    }
}

fn add_matched_subscriptions_detector(
    writer: &mut DdsDataWriter<RtpsStatefulWriter>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
    {
        let remote_reader_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let expects_inline_qos = false;
        let proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            expects_inline_qos,
            true,
            ReliabilityKind::Reliable,
            DurabilityKind::TransientLocal,
        );
        writer.matched_reader_add(proxy);
    }
}

fn add_matched_publications_announcer(
    reader: &mut DdsDataReader<RtpsStatefulReader>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
    {
        let remote_writer_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let data_max_size_serialized = None;

        let proxy = RtpsWriterProxy::new(
            remote_writer_guid,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            data_max_size_serialized,
            remote_group_entity_id,
        );

        reader.matched_writer_add(proxy);
    }
}

fn add_matched_publications_detector(
    writer: &mut DdsDataWriter<RtpsStatefulWriter>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
    {
        let remote_reader_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let expects_inline_qos = false;
        let proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            expects_inline_qos,
            true,
            ReliabilityKind::Reliable,
            DurabilityKind::TransientLocal,
        );
        writer.matched_reader_add(proxy);
    }
}

fn add_matched_topics_announcer(
    reader: &mut DdsDataReader<RtpsStatefulReader>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
    {
        let remote_writer_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let data_max_size_serialized = None;

        let proxy = RtpsWriterProxy::new(
            remote_writer_guid,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            data_max_size_serialized,
            remote_group_entity_id,
        );
        reader.matched_writer_add(proxy);
    }
}

fn add_matched_topics_detector(
    writer: &mut DdsDataWriter<RtpsStatefulWriter>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
    {
        let remote_reader_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let expects_inline_qos = false;
        let proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            expects_inline_qos,
            true,
            ReliabilityKind::Reliable,
            DurabilityKind::TransientLocal,
        );
        writer.matched_reader_add(proxy);
    }
}

pub fn receive_builtin_message(
    domain_participant: &mut DdsDomainParticipant,
    message: RtpsMessage,
    locator: Locator,
    sedp_condvar: &DdsCondvar,
    listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
) {
    domain_participant
        .receive_builtin_data(locator, message, listener_sender)
        .ok();

    discover_matched_participants(domain_participant, sedp_condvar).ok();
    domain_participant
        .discover_matched_readers(listener_sender)
        .ok();
    discover_matched_writers(domain_participant, listener_sender).ok();
    domain_participant
        .discover_matched_topics(listener_sender)
        .ok();
}

fn send_user_defined_message(
    domain_participant: &mut DdsDomainParticipant,
    metatraffic_unicast_transport_send: &mut impl TransportWrite,
) {
    let header = RtpsMessageHeader {
        protocol: ProtocolId::PROTOCOL_RTPS,
        version: domain_participant.protocol_version(),
        vendor_id: domain_participant.vendor_id(),
        guid_prefix: domain_participant.guid().prefix(),
    };

    let _now = domain_participant.get_current_time();
    stateless_writer_send_message(
        domain_participant
            .get_builtin_publisher_mut()
            .stateless_data_writer_list_mut()
            .iter_mut()
            .find(|x| x.get_type_name() == SpdpDiscoveredParticipantData::type_name())
            .unwrap(),
        header,
        metatraffic_unicast_transport_send,
    );

    user_defined_stateful_writer_send_message(
        domain_participant
            .get_builtin_publisher_mut()
            .stateful_data_writer_list_mut()
            .iter_mut()
            .find(|x| x.get_type_name() == DiscoveredWriterData::type_name())
            .unwrap(),
        header,
        metatraffic_unicast_transport_send,
    );

    user_defined_stateful_writer_send_message(
        domain_participant
            .get_builtin_publisher_mut()
            .stateful_data_writer_list_mut()
            .iter_mut()
            .find(|x| x.get_type_name() == DiscoveredReaderData::type_name())
            .unwrap(),
        header,
        metatraffic_unicast_transport_send,
    );

    user_defined_stateful_writer_send_message(
        domain_participant
            .get_builtin_publisher_mut()
            .stateful_data_writer_list_mut()
            .iter_mut()
            .find(|x| x.get_type_name() == DiscoveredTopicData::type_name())
            .unwrap(),
        header,
        metatraffic_unicast_transport_send,
    );

    for stateful_readers in domain_participant
        .get_builtin_subscriber_mut()
        .stateful_data_reader_list_mut()
    {
        stateful_readers.send_message(header, metatraffic_unicast_transport_send)
    }
}

fn announce_entity(domain_participant: &mut DdsDomainParticipant, announce_kind: AnnounceKind) {
    match announce_kind {
        AnnounceKind::CreatedDataReader(discovered_reader_data) => {
            announce_created_data_reader(domain_participant, discovered_reader_data)
        }
        AnnounceKind::CreatedDataWriter(discovered_writer_data) => {
            announce_created_data_writer(domain_participant, discovered_writer_data)
        }
        AnnounceKind::CratedTopic(discovered_topic_data) => {
            announce_created_topic(domain_participant, discovered_topic_data)
        }
        AnnounceKind::DeletedDataReader(deleted_reader_handle) => {
            announce_deleted_reader(domain_participant, deleted_reader_handle)
        }
        AnnounceKind::DeletedDataWriter(deleted_writer_handle) => {
            announce_deleted_writer(domain_participant, deleted_writer_handle)
        }
        AnnounceKind::DeletedParticipant => (),
    }
}

fn user_defined_communication_send(
    domain_participant: &mut DdsDomainParticipant,
    default_unicast_transport_send: &mut impl TransportWrite,
) {
    let header = RtpsMessageHeader {
        protocol: ProtocolId::PROTOCOL_RTPS,
        version: domain_participant.protocol_version(),
        vendor_id: domain_participant.vendor_id(),
        guid_prefix: domain_participant.guid().prefix(),
    };
    let now = domain_participant.get_current_time();

    for publisher in domain_participant.user_defined_publisher_list_mut() {
        for data_writer in publisher.stateful_data_writer_list_mut() {
            let writer_id = data_writer.guid().entity_id();
            let data_max_size_serialized = data_writer.data_max_size_serialized();
            let heartbeat_period = data_writer.heartbeat_period();
            let first_sn = data_writer
                .change_list()
                .iter()
                .map(|x| x.sequence_number())
                .min()
                .unwrap_or(SequenceNumber::new(1));
            let last_sn = data_writer
                .change_list()
                .iter()
                .map(|x| x.sequence_number())
                .max()
                .unwrap_or_else(|| SequenceNumber::new(0));
            remove_stale_writer_changes(data_writer, now);
            for reader_proxy in &mut data_writer.matched_reader_list() {
                match reader_proxy.reliability() {
                    ReliabilityKind::BestEffort => send_message_best_effort_reader_proxy(
                        reader_proxy,
                        data_max_size_serialized,
                        header,
                        default_unicast_transport_send,
                    ),
                    ReliabilityKind::Reliable => send_message_reliable_reader_proxy(
                        reader_proxy,
                        data_max_size_serialized,
                        header,
                        default_unicast_transport_send,
                        writer_id,
                        first_sn,
                        last_sn,
                        heartbeat_period,
                    ),
                }
            }
        }
    }

    for subscriber in domain_participant.user_defined_subscriber_list_mut() {
        for data_reader in subscriber.stateful_data_reader_list_mut() {
            data_reader.send_message(header, default_unicast_transport_send)
        }
    }
}
