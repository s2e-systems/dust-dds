use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    str::FromStr,
    sync::Mutex,
};

use rust_dds_api::{
    builtin_topics::ParticipantBuiltinTopicData,
    dcps_psm::{BuiltInTopicKey, DomainId, StatusMask, Time},
    domain::{
        domain_participant::DomainParticipant,
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::qos::{
        DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
        PublisherQos, SubscriberQos,
    },
    publication::{data_writer::DataWriter, publisher::PublisherDataWriterFactory},
    return_type::{DDSError, DDSResult},
    subscription::subscriber::SubscriberDataReaderFactory,
};
use rust_dds_rtps_implementation::{
    data_representation_builtin_endpoints::{
        sedp_discovered_reader_data::{SedpDiscoveredReaderData, DCPS_SUBSCRIPTION},
        sedp_discovered_topic_data::{SedpDiscoveredTopicData, DCPS_TOPIC},
        sedp_discovered_writer_data::{SedpDiscoveredWriterData, DCPS_PUBLICATION},
        spdp_discovered_participant_data::{
            ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
        },
    },
    dds_impl::{
        data_reader_proxy::{DataReaderAttributes, RtpsReader},
        data_writer_proxy::{DataWriterAttributes, DataWriterProxy, RtpsWriter},
        domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
        publisher_proxy::{PublisherAttributes, PublisherProxy},
        subscriber_proxy::{SubscriberAttributes, SubscriberProxy},
        topic_proxy::TopicAttributes,
    },
    dds_type::DdsType,
    rtps_impl::{
        rtps_group_impl::RtpsGroupImpl, rtps_participant_impl::RtpsParticipantImpl,
        rtps_reader_locator_impl::RtpsReaderLocatorAttributesImpl,
        rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_reader_impl::RtpsStatelessReaderImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
    },
    utils::{rtps_structure::RtpsStructure, shared_object::RtpsShared},
};
use rust_rtps_pim::{
    behavior::writer::{
        reader_locator::RtpsReaderLocatorConstructor,
        stateless_writer::RtpsStatelessWriterOperations,
    },
    discovery::{
        sedp::builtin_endpoints::{
            SedpBuiltinPublicationsReader, SedpBuiltinPublicationsWriter,
            SedpBuiltinSubscriptionsReader, SedpBuiltinSubscriptionsWriter,
            SedpBuiltinTopicsReader, SedpBuiltinTopicsWriter,
        },
        spdp::builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    structure::{
        entity::RtpsEntityAttributes,
        group::RtpsGroupConstructor,
        participant::RtpsParticipantAttributes,
        types::{
            EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, BUILT_IN_READER_GROUP,
            BUILT_IN_WRITER_GROUP, PROTOCOLVERSION, VENDOR_ID_S2E,
        },
    },
};
use socket2::Socket;

use crate::{
    communication::Communication,
    tasks::{task_sedp_writer_discovery, task_sedp_reader_discovery, task_spdp_discovery, Executor, Spawner},
    udp_transport::UdpTransport,
};

pub struct RtpsStructureImpl;

impl RtpsStructure for RtpsStructureImpl {
    type Group = RtpsGroupImpl;
    type Participant = RtpsParticipantImpl;
    type StatelessWriter = RtpsStatelessWriterImpl;
    type StatefulWriter = RtpsStatefulWriterImpl;
    type StatelessReader = RtpsStatelessReaderImpl;
    type StatefulReader = RtpsStatefulReaderImpl;
}

/// The DomainParticipant object plays several roles:
/// - It acts as a container for all other Entity objects.
/// - It acts as factory for the Publisher, Subscriber, Topic, and MultiTopic Entity objects.
/// - It represents the participation of the application on a communication plane that isolates applications running on the
/// same set of physical computers from each other. A domain establishes a “virtual network” linking all applications that
/// share the same domainId and isolating them from applications running on different domains. In this way, several
/// independent distributed applications can coexist in the same physical network without interfering, or even being aware
/// of each other.
/// - It provides administration services in the domain, offering operations that allow the application to ‘ignore’ locally any
/// information about a given participant (ignore_participant), publication (ignore_publication), subscription
/// (ignore_subscription), or topic (ignore_topic).
///
/// The following sub clauses explain all the operations in detail.
/// The following operations may be called even if the DomainParticipant is not enabled. Other operations will have the value
/// NOT_ENABLED if called on a disabled DomainParticipant:
/// - Operations defined at the base-class level namely, set_qos, get_qos, set_listener, get_listener, and enable.
/// - Factory methods: create_topic, create_publisher, create_subscriber, delete_topic, delete_publisher,
/// delete_subscriber
/// - Operations that access the status: get_statuscondition

const PB: u16 = 7400;
const DG: u16 = 250;
const PG: u16 = 2;
#[allow(non_upper_case_globals)]
const d0: u16 = 0;
#[allow(non_upper_case_globals)]
const d1: u16 = 10;
#[allow(non_upper_case_globals)]
const _d2: u16 = 1;
#[allow(non_upper_case_globals)]
const d3: u16 = 11;

fn get_builtin_multicast_socket(domain_id: u16) -> Option<UdpSocket> {
    let socket_addr = SocketAddr::from(([127, 0, 0, 1], PB + DG * domain_id + d0));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .ok()?;
    socket.set_reuse_address(true).ok()?;
    socket.set_nonblocking(true).ok()?;
    socket.bind(&socket_addr.into()).ok()?;
    socket
        .join_multicast_v4(
            &Ipv4Addr::from_str("239.255.0.1").unwrap(),
            &Ipv4Addr::from_str("127.0.0.1").unwrap(),
        )
        .ok()?;
    socket.set_multicast_loop_v4(true).ok()?;

    Some(socket.into())
}

fn get_builtin_unicast_socket(domain_id: u16, participant_id: u16) -> Option<UdpSocket> {
    let socket_addr = SocketAddr::from((
        [127, 0, 0, 1],
        PB + DG * domain_id + d1 + PG * participant_id,
    ));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .ok()?;
    socket.set_nonblocking(true).ok()?;
    socket.bind(&socket_addr.into()).ok()?;

    Some(socket.into())
}

fn _get_user_defined_multicast_socket(domain_id: u16) -> Option<UdpSocket> {
    let socket_addr = SocketAddr::from(([127, 0, 0, 1], PB + DG * domain_id + _d2));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .ok()?;
    socket.set_reuse_address(true).ok()?;
    socket.set_nonblocking(true).ok()?;
    socket.bind(&socket_addr.into()).ok()?;
    socket
        .join_multicast_v4(
            &Ipv4Addr::from_str("239.255.0.1").unwrap(),
            &Ipv4Addr::from_str("127.0.0.1").unwrap(),
        )
        .ok()?;
    socket.set_multicast_loop_v4(true).ok()?;

    Some(socket.into())
}

fn get_user_defined_unicast_socket(domain_id: u16, participant_id: u16) -> Option<UdpSocket> {
    let socket_addr = SocketAddr::from((
        [127, 0, 0, 1],
        PB + DG * domain_id + d3 + PG * participant_id,
    ));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .ok()?;
    socket.set_nonblocking(true).ok()?;
    socket.bind(&socket_addr.into()).ok()?;

    Some(socket.into())
}

pub struct DomainParticipantFactory {
    participant_list: Mutex<Vec<RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>>>,
}

impl DomainParticipantFactory {
    /// This operation creates a new DomainParticipant object. The DomainParticipant signifies that the calling application intends
    /// to join the Domain identified by the domain_id argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no DomainParticipant will be created.
    /// The special value PARTICIPANT_QOS_DEFAULT can be used to indicate that the DomainParticipant should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation get_default_participant_qos (2.2.2.2.2.6) and using the resulting
    /// QoS to create the DomainParticipant.
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    ///
    /// Developer note: Ideally this method should return impl DomainParticipant. However because of the GAT workaround used there is no way
    /// to call,e.g. create_topic(), because we can't write impl DomainParticipant + for<'t, T> TopicGAT<'t, T> on the return. This issue will
    /// probably be solved once the GAT functionality is available on stable.
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: StatusMask,
    ) -> DDSResult<DomainParticipantProxy<RtpsStructureImpl>> {
        let guid_prefix = GuidPrefix([3; 12]);
        let qos = qos.unwrap_or_default();

        let domain_participant = RtpsShared::new(DomainParticipantAttributes::new(
            guid_prefix,
            domain_id,
            "".to_string(),
            qos.clone(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                7400,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                7400,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                7410,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
            )],
            vec![],
        ));

        create_builtins(guid_prefix, domain_participant.clone())?;

        if qos.entity_factory.autoenable_created_entities {
            self.enable(domain_participant.clone())?;
        }

        self.participant_list
            .lock()
            .unwrap()
            .push(domain_participant.clone());

        Ok(DomainParticipantProxy::new(domain_participant.downgrade()))
    }

    pub fn enable(
        &self,
        domain_participant: RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
    ) -> DDSResult<()> {
        let guid_prefix = domain_participant
            .read_lock()
            .rtps_participant
            .guid()
            .prefix;
        let domain_id = domain_participant.read_lock().domain_id;
        let domain_tag = domain_participant.read_lock().domain_tag.clone();
        let participant_id = self.participant_list.lock().unwrap().len();

        // ////////// Task creation
        let (executor, spawner) = {
            let (sender, receiver) = std::sync::mpsc::sync_channel(10);
            (Executor { receiver }, Spawner::new(sender))
        };

        {
            let builtin_unicast_transport = UdpTransport::new(
                get_builtin_unicast_socket(domain_id as u16, participant_id as u16).unwrap()
            );
            let mut communication = Communication {
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix,
                transport: builtin_unicast_transport,
            };

            let domain_participant = domain_participant.clone();
            spawner.spawn_enabled_periodic_task(
                "builtin sedp communication",
                move || {
                    let builtin_publisher = &domain_participant.read_lock().builtin_publisher;
                    let builtin_subscriber = &domain_participant.read_lock().builtin_subscriber;

                    if let (Some(builtin_publisher), Some(builtin_subscriber)) =
                        (builtin_publisher, builtin_subscriber)
                    {
                        communication.send(core::slice::from_ref(builtin_publisher));
                        communication.receive(core::slice::from_ref(builtin_subscriber));
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        {
            let builtin_multicast_transport = UdpTransport::new(
                get_builtin_multicast_socket(domain_id as u16).unwrap()
            );
            let mut communication = Communication {
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix,
                transport: builtin_multicast_transport,
            };

            let domain_participant = domain_participant.clone();
            spawner.spawn_enabled_periodic_task(
                "builtin spdp communication",
                move || {
                    let builtin_subscriber = &domain_participant.read_lock().builtin_subscriber;

                    if let Some(builtin_subscriber) = builtin_subscriber
                    {
                        communication.receive(core::slice::from_ref(builtin_subscriber));
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        {
            let user_unicast_transport = UdpTransport::new(
                get_user_defined_unicast_socket(domain_id as u16, participant_id as u16).unwrap()
            );
            let mut communication = Communication {
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix,
                transport: user_unicast_transport,
            };

            let domain_participant = domain_participant.clone();
            spawner.spawn_enabled_periodic_task(
                "user-defined communication",
                move || {
                    communication.send(
                        domain_participant
                            .read_lock()
                            .user_defined_publisher_list
                            .as_ref(),
                    );
                    communication.receive(
                        domain_participant
                            .read_lock()
                            .user_defined_subscriber_list
                            .as_ref(),
                    );
                },
                std::time::Duration::from_millis(500),
            );
        }

        {
            let domain_participant = domain_participant.clone();

            let try_perform_task = move || -> Option<()> {
                let participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());
                let builtin_subscriber = SubscriberProxy::new(
                    participant_proxy.clone(),
                    domain_participant
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()?
                        .downgrade(),
                );
                let builtin_publisher = PublisherProxy::new(
                    domain_participant
                        .read_lock()
                        .builtin_publisher
                        .as_ref()?
                        .downgrade(),
                );

                let participant_topic = participant_proxy
                    .lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT).ok()?;
                let publication_topic = participant_proxy
                    .lookup_topicdescription::<SedpDiscoveredWriterData>(DCPS_PUBLICATION).ok()?;
                let subscription_topic = participant_proxy
                    .lookup_topicdescription::<SedpDiscoveredReaderData>(DCPS_SUBSCRIPTION).ok()?;
                let topic_topic =
                    participant_proxy.lookup_topicdescription::<SedpDiscoveredTopicData>(DCPS_TOPIC).ok()?;

                let mut builtin_participant_data_reader =
                    builtin_subscriber.datareader_factory_lookup_datareader(&participant_topic).ok()?;

                let builtin_publication_reader =
                    builtin_subscriber.datareader_factory_lookup_datareader(&publication_topic).ok()?;
                let builtin_subscription_reader =
                    builtin_subscriber.datareader_factory_lookup_datareader(&subscription_topic).ok()?;
                let builtin_topic_reader =
                    builtin_subscriber.datareader_factory_lookup_datareader(&topic_topic).ok()?;
                let builtin_publication_writer =
                    builtin_publisher.datawriter_factory_lookup_datawriter(&publication_topic).ok()?;
                let builtin_subscription_writer =
                    builtin_publisher.datawriter_factory_lookup_datawriter(&subscription_topic).ok()?;
                let builtin_topic_writer =
                    builtin_publisher.datawriter_factory_lookup_datawriter(&topic_topic).ok()?;

                task_spdp_discovery(
                    &mut builtin_participant_data_reader,
                    domain_id as u32,
                    &domain_tag,
                    builtin_publication_writer
                        .as_ref()
                        .upgrade()
                        .ok()?
                        .write_lock()
                        .rtps_writer
                        .try_as_stateful_writer()
                        .ok()?,
                    builtin_publication_reader
                        .as_ref()
                        .upgrade()
                        .ok()?
                        .write_lock()
                        .rtps_reader
                        .try_as_stateful_reader()
                        .ok()?,
                    builtin_subscription_writer
                        .as_ref()
                        .upgrade()
                        .ok()?
                        .write_lock()
                        .rtps_writer
                        .try_as_stateful_writer()
                        .ok()?,
                    builtin_subscription_reader
                        .as_ref()
                        .upgrade()
                        .ok()?
                        .write_lock()
                        .rtps_reader
                        .try_as_stateful_reader()
                        .ok()?,
                    builtin_topic_writer
                        .as_ref()
                        .upgrade()
                        .ok()?
                        .write_lock()
                        .rtps_writer
                        .try_as_stateful_writer()
                        .ok()?,
                    builtin_topic_reader
                        .as_ref()
                        .upgrade()
                        .ok()?
                        .write_lock()
                        .rtps_reader
                        .try_as_stateful_reader()
                        .ok()?,
                );

                Some(())
            };

            spawner.spawn_enabled_periodic_task(
                "spdp discovery",
                move || {
                    match try_perform_task() {
                        Some(()) => (),
                        None     => println!("spdp discovery failed (domain participant might not be fully constructed yet)")
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        // {
        //     let domain_participant = domain_participant.clone();

        //     spawner.spawn_enabled_periodic_task(
        //         "sedp discovery",
        //         move || {
        //             let user_defined_publisher_list = domain_participant.write_lock().user_defined_publisher_list;
        //             for user_defined_publisher in user_defined_publisher_list.iter() {
        //                 user_defined_publisher.process_discovery();
        //             }
        //         },
        //         std::time::Duration::from_millis(500),
        //     );
        // }

        {
            let domain_participant = domain_participant.clone();

            let try_perform_task = move || -> Option<()> {
                let participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());
                let builtin_subscriber = SubscriberProxy::new(
                    participant_proxy.clone(),
                    domain_participant
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()?
                        .downgrade(),
                );

                let publication_topic = participant_proxy
                    .lookup_topicdescription::<SedpDiscoveredWriterData>(DCPS_PUBLICATION).ok()?;
                let mut builtin_publication_reader =
                    builtin_subscriber.datareader_factory_lookup_datareader(&publication_topic).ok()?;

                let subscription_topic = participant_proxy
                    .lookup_topicdescription::<SedpDiscoveredReaderData>(DCPS_SUBSCRIPTION).ok()?;
                let mut builtin_subscription_reader =
                    builtin_subscriber.datareader_factory_lookup_datareader(&subscription_topic).ok()?;

                task_sedp_writer_discovery(
                    &mut builtin_publication_reader,
                    &domain_participant.read_lock().user_defined_subscriber_list,
                );

                task_sedp_reader_discovery(
                    &mut builtin_subscription_reader,
                    &domain_participant.read_lock().user_defined_publisher_list,
                );

                Some(())
            };

            spawner.spawn_enabled_periodic_task(
                "sedp discovery",
                move || {
                    match try_perform_task() {
                        Some(()) => (),
                        None     => println!("sedp discovery failed (domain participant might not be fully constructed yet)")
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        let spdp_discovered_participant_data =
            spdp_discovered_participant_data_from_domain_participant(
                &domain_participant.read_lock(),
            );

        let builtin_participant_data_writer = domain_participant
            .read_lock()
            .builtin_publisher
            .as_ref()
            .ok_or(DDSError::PreconditionNotMet(
                "No builtin publisher".to_string(),
            ))?
            .read_lock()
            .data_writer_list
            .iter()
            .find(|w| w.read_lock().topic.read_lock().topic_name == DCPS_PARTICIPANT)
            .ok_or(DDSError::PreconditionNotMet(
                "No builtin participant data writer".to_string(),
            ))?
            .clone();

        DataWriterProxy::new(builtin_participant_data_writer.downgrade()).write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            Time { sec: 0, nanosec: 0 },
        )?;

        spawner.enable_tasks();
        executor.run();

        Ok(())
    }

    /// This operation deletes an existing DomainParticipant. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted. Otherwise the error PRECONDITION_NOT_MET is returned.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_participant(
        &self,
        _a_participant: DomainParticipantProxy<RtpsStructureImpl>,
    ) -> DDSResult<()> {
        todo!()
    }

    /// This operation returns the DomainParticipantFactory singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same DomainParticipantFactory instance.
    /// The get_instance operation is a static operation implemented using the syntax of the native language and can therefore not be
    /// expressed in the IDL PSM.
    /// The pre-defined value TheParticipantFactory can also be used as an alias for the singleton factory returned by the operation
    /// get_instance.
    pub fn get_instance() -> Self {
        Self {
            participant_list: Mutex::new(Vec::new()),
        }
    }

    /// This operation retrieves a previously created DomainParticipant belonging to specified domain_id. If no such
    /// DomainParticipant exists, the operation will return a ‘nil’ value.
    /// If multiple DomainParticipant entities belonging to that domain_id exist, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_participant(
        &self,
        _domain_id: DomainId,
    ) -> DomainParticipantProxy<RtpsStructureImpl> {
        todo!()
    }

    /// This operation sets a default value of the DomainParticipant QoS policies which will be used for newly created
    /// DomainParticipant entities in the case where the QoS policies are defaulted in the create_participant operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    pub fn set_default_participant_qos(&self, _qos: DomainParticipantQos) -> DDSResult<()> {
        todo!()
    }

    /// This operation retrieves the default value of the DomainParticipant QoS, that is, the QoS policies which will be used for
    /// newly created DomainParticipant entities in the case where the QoS policies are defaulted in the create_participant
    /// operation.
    /// The values retrieved get_default_participant_qos will match the set of values specified on the last successful call to
    /// set_default_participant_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3,
    /// Supported QoS.
    pub fn get_default_participant_qos(&self) -> DDSResult<DomainParticipantQos> {
        todo!()
    }

    /// This operation sets the value of the DomainParticipantFactory QoS policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the DomainParticipantFactory is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    pub fn set_qos(&self, _qos: DomainParticipantFactoryQos) -> DDSResult<()> {
        todo!()
    }

    /// This operation returns the value of the DomainParticipantFactory QoS policies.
    pub fn get_qos(&self) -> DomainParticipantFactoryQos {
        todo!()
    }
}

pub fn spdp_discovered_participant_data_from_domain_participant<Rtps>(
    participant: &DomainParticipantAttributes<Rtps>,
) -> SpdpDiscoveredParticipantData
where
    Rtps: RtpsStructure,
    Rtps::Participant: RtpsParticipantAttributes,
{
    SpdpDiscoveredParticipantData {
        dds_participant_data: ParticipantBuiltinTopicData {
            key: BuiltInTopicKey {
                value: (*participant.rtps_participant.guid()).into(),
            },
            user_data: participant.qos.user_data.clone(),
        },
        participant_proxy: ParticipantProxy {
            domain_id: participant.domain_id as u32,
            domain_tag: participant.domain_tag.clone(),
            protocol_version: *participant.rtps_participant.protocol_version(),
            guid_prefix: *participant.rtps_participant.guid().prefix(),
            vendor_id: *participant.rtps_participant.vendor_id(),
            expects_inline_qos: false,
            metatraffic_unicast_locator_list: participant.metatraffic_unicast_locator_list.clone(),
            metatraffic_multicast_locator_list: participant
                .metatraffic_multicast_locator_list
                .clone(),
            default_unicast_locator_list: participant
                .rtps_participant
                .default_unicast_locator_list()
                .to_vec(),
            default_multicast_locator_list: participant
                .rtps_participant
                .default_multicast_locator_list()
                .to_vec(),
            available_builtin_endpoints: BuiltinEndpointSet::default(),
            manual_liveliness_count: participant.manual_liveliness_count,
            builtin_endpoint_qos: BuiltinEndpointQos::default(),
        },
        lease_duration: participant.lease_duration,
    }
}

fn create_builtins(
    guid_prefix: GuidPrefix,
    domain_participant: RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DDSResult<()> {
    // ///////// Create the built-in publisher and subcriber

    let builtin_subscriber = RtpsShared::new(SubscriberAttributes::new(
        SubscriberQos::default(),
        RtpsGroupImpl::new(Guid::new(
            guid_prefix,
            EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
        )),
        domain_participant.downgrade(),
    ));
    domain_participant.write_lock().builtin_subscriber = Some(builtin_subscriber.clone());

    let builtin_publisher = RtpsShared::new(PublisherAttributes::new(
        PublisherQos::default(),
        RtpsGroupImpl::new(Guid::new(
            guid_prefix,
            EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
        )),
        domain_participant.downgrade(),
    ));
    domain_participant.write_lock().builtin_subscriber = Some(builtin_subscriber.clone());
    domain_participant.write_lock().builtin_publisher = Some(builtin_publisher.clone());

    // ///////// Create built-in DDS data readers and data writers

    // ////////// SPDP built-in topic, reader and writer
    {
        let spdp_topic_participant = RtpsShared::new(TopicAttributes::new(
            domain_participant.read_lock().default_topic_qos.clone(),
            SpdpDiscoveredParticipantData::type_name(),
            DCPS_PARTICIPANT,
            domain_participant.downgrade(),
        ));
        domain_participant
            .write_lock()
            .topic_list
            .push(spdp_topic_participant.clone());

        let spdp_builtin_participant_rtps_reader =
            SpdpBuiltinParticipantReader::create::<RtpsStatelessReaderImpl>(guid_prefix, &[], &[]);

        let spdp_builtin_participant_data_reader = RtpsShared::new(DataReaderAttributes::new(
            DataReaderQos::default(),
            RtpsReader::Stateless(spdp_builtin_participant_rtps_reader),
            spdp_topic_participant.clone(),
            builtin_subscriber.downgrade(),
        ));
        builtin_subscriber
            .write_lock()
            .data_reader_list
            .push(spdp_builtin_participant_data_reader.clone());

        let mut spdp_builtin_participant_rtps_writer =
            SpdpBuiltinParticipantWriter::create::<RtpsStatelessWriterImpl>(guid_prefix, &[], &[]);

        let spdp_discovery_locator = RtpsReaderLocatorAttributesImpl::new(
            Locator::new(
                LOCATOR_KIND_UDPv4,
                7400,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
            ),
            false,
        );

        spdp_builtin_participant_rtps_writer.reader_locator_add(spdp_discovery_locator);

        let spdp_builtin_participant_data_writer = RtpsShared::new(DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateless(spdp_builtin_participant_rtps_writer),
            spdp_topic_participant.clone(),
            builtin_publisher.downgrade(),
        ));
        builtin_publisher
            .write_lock()
            .data_writer_list
            .push(spdp_builtin_participant_data_writer.clone());
    }

    // ////////// SEDP built-in publication topic, reader and writer
    {
        let sedp_topic_publication = RtpsShared::new(TopicAttributes::new(
            domain_participant.read_lock().default_topic_qos.clone(),
            SedpDiscoveredWriterData::type_name(),
            DCPS_PUBLICATION,
            domain_participant.downgrade(),
        ));
        domain_participant
            .write_lock()
            .topic_list
            .push(sedp_topic_publication.clone());

        let sedp_builtin_publications_rtps_reader =
            SedpBuiltinPublicationsReader::create::<RtpsStatefulReaderImpl>(guid_prefix, &[], &[]);
        let sedp_builtin_publications_data_reader = RtpsShared::new(DataReaderAttributes::new(
            DataReaderQos::default(),
            RtpsReader::Stateful(sedp_builtin_publications_rtps_reader),
            sedp_topic_publication.clone(),
            builtin_subscriber.downgrade(),
        ));
        builtin_subscriber
            .write_lock()
            .data_reader_list
            .push(sedp_builtin_publications_data_reader.clone());

        let sedp_builtin_publications_rtps_writer =
            SedpBuiltinPublicationsWriter::create::<RtpsStatefulWriterImpl>(guid_prefix, &[], &[]);
        let sedp_builtin_publications_data_writer = RtpsShared::new(DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateful(sedp_builtin_publications_rtps_writer),
            sedp_topic_publication.clone(),
            builtin_publisher.downgrade(),
        ));
        builtin_publisher
            .write_lock()
            .data_writer_list
            .push(sedp_builtin_publications_data_writer.clone());
    }

    // ////////// SEDP built-in subcriptions topic, reader and writer
    {
        let sedp_topic_subscription = RtpsShared::new(TopicAttributes::new(
            domain_participant.read_lock().default_topic_qos.clone(),
            SedpDiscoveredReaderData::type_name(),
            DCPS_SUBSCRIPTION,
            domain_participant.downgrade(),
        ));
        domain_participant
            .write_lock()
            .topic_list
            .push(sedp_topic_subscription.clone());

        let sedp_builtin_subscriptions_rtps_reader =
            SedpBuiltinSubscriptionsReader::create::<RtpsStatefulReaderImpl>(guid_prefix, &[], &[]);
        let sedp_builtin_subscriptions_data_reader = RtpsShared::new(DataReaderAttributes::new(
            DataReaderQos::default(),
            RtpsReader::Stateful(sedp_builtin_subscriptions_rtps_reader),
            sedp_topic_subscription.clone(),
            builtin_subscriber.downgrade(),
        ));
        builtin_subscriber
            .write_lock()
            .data_reader_list
            .push(sedp_builtin_subscriptions_data_reader.clone());

        let sedp_builtin_subscriptions_rtps_writer =
            SedpBuiltinSubscriptionsWriter::create::<RtpsStatefulWriterImpl>(guid_prefix, &[], &[]);
        let sedp_builtin_subscriptions_data_writer = RtpsShared::new(DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateful(sedp_builtin_subscriptions_rtps_writer),
            sedp_topic_subscription.clone(),
            builtin_publisher.downgrade(),
        ));
        builtin_publisher
            .write_lock()
            .data_writer_list
            .push(sedp_builtin_subscriptions_data_writer.clone());
    }

    // ////////// SEDP built-in topics topic, reader and writer
    {
        let sedp_topic_topic = RtpsShared::new(TopicAttributes::new(
            domain_participant.read_lock().default_topic_qos.clone(),
            SedpDiscoveredTopicData::type_name(),
            DCPS_TOPIC,
            domain_participant.downgrade(),
        ));
        domain_participant
            .write_lock()
            .topic_list
            .push(sedp_topic_topic.clone());

        let sedp_builtin_topics_rtps_reader =
            SedpBuiltinTopicsReader::create::<RtpsStatefulReaderImpl>(guid_prefix, &[], &[]);
        let sedp_builtin_topics_data_reader = RtpsShared::new(DataReaderAttributes::new(
            DataReaderQos::default(),
            RtpsReader::Stateful(sedp_builtin_topics_rtps_reader),
            sedp_topic_topic.clone(),
            builtin_subscriber.downgrade(),
        ));
        builtin_subscriber
            .write_lock()
            .data_reader_list
            .push(sedp_builtin_topics_data_reader.clone());

        let sedp_builtin_topics_rtps_writer =
            SedpBuiltinTopicsWriter::create::<RtpsStatefulWriterImpl>(guid_prefix, &[], &[]);
        let sedp_builtin_topics_data_writer = RtpsShared::new(DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateful(sedp_builtin_topics_rtps_writer),
            sedp_topic_topic.clone(),
            builtin_publisher.downgrade(),
        ));
        builtin_publisher
            .write_lock()
            .data_writer_list
            .push(sedp_builtin_topics_data_writer.clone());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use rust_dds_api::{
        dcps_psm::DomainId,
        domain::domain_participant::DomainParticipant,
        infrastructure::qos::DomainParticipantQos,
        publication::publisher::PublisherDataWriterFactory,
        subscription::subscriber::SubscriberDataReaderFactory,
    };
    use rust_dds_rtps_implementation::{
        data_representation_builtin_endpoints::{
            sedp_discovered_reader_data::SedpDiscoveredReaderData,
            sedp_discovered_topic_data::SedpDiscoveredTopicData,
            sedp_discovered_writer_data::SedpDiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        dds_impl::{
            domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
            publisher_proxy::PublisherProxy,
            subscriber_proxy::SubscriberProxy,
        },
        utils::shared_object::RtpsShared,
    };
    use rust_rtps_pim::structure::types::GuidPrefix;

    use super::{
        create_builtins, DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC, get_builtin_multicast_socket, PB, d0,
    };

    #[test]
    fn multicast_socket_behaviour() {
        let multicast_addr = SocketAddr::from(([239, 255, 0, 1], PB + d0));

        let socket1 = get_builtin_multicast_socket(0).unwrap();
        let socket2 = get_builtin_multicast_socket(0).unwrap();
        let socket3 = get_builtin_multicast_socket(0).unwrap();

        socket1.send_to(&[1, 2, 3, 4], multicast_addr)
            .unwrap();
        
        // Everyone receives the data
        let mut buf = [0; 4];
        let (size, _) = socket1.recv_from(&mut buf).unwrap();
        assert_eq!(4, size);
        let (size, _) = socket2.recv_from(&mut buf).unwrap();
        assert_eq!(4, size);
        let (size, _) = socket3.recv_from(&mut buf).unwrap();
        assert_eq!(4, size);

        // Data is received only once
        assert!(socket1.recv_from(&mut buf).is_err());
        assert!(socket2.recv_from(&mut buf).is_err());
        assert!(socket3.recv_from(&mut buf).is_err());
    }

    #[test]
    fn create_builtins_adds_builtin_readers_and_writers() {
        let guid_prefix = GuidPrefix([0; 12]);
        let domain_participant = RtpsShared::new(DomainParticipantAttributes::new(
            guid_prefix,
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        create_builtins(guid_prefix, domain_participant.clone()).unwrap();

        let participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

        let participant_topic = participant_proxy
            .lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
            .unwrap();
        let publication_topic = participant_proxy
            .lookup_topicdescription::<SedpDiscoveredWriterData>(DCPS_PUBLICATION)
            .unwrap();
        let subscription_topic = participant_proxy
            .lookup_topicdescription::<SedpDiscoveredReaderData>(DCPS_SUBSCRIPTION)
            .unwrap();
        let topic_topic = participant_proxy
            .lookup_topicdescription::<SedpDiscoveredTopicData>(DCPS_TOPIC)
            .unwrap();

        let builtin_subscriber = SubscriberProxy::new(
            participant_proxy,
            domain_participant
                .read_lock()
                .builtin_subscriber
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let builtin_publisher = PublisherProxy::new(
            domain_participant
                .read_lock()
                .builtin_publisher
                .as_ref()
                .unwrap()
                .downgrade(),
        );

        assert!(builtin_subscriber
            .datareader_factory_lookup_datareader(&participant_topic)
            .is_ok());
        assert!(builtin_subscriber
            .datareader_factory_lookup_datareader(&publication_topic)
            .is_ok());
        assert!(builtin_subscriber
            .datareader_factory_lookup_datareader(&subscription_topic)
            .is_ok());
        assert!(builtin_subscriber
            .datareader_factory_lookup_datareader(&topic_topic)
            .is_ok());

        assert!(builtin_publisher
            .datawriter_factory_lookup_datawriter(&participant_topic)
            .is_ok());
        assert!(builtin_publisher
            .datawriter_factory_lookup_datawriter(&publication_topic)
            .is_ok());
        assert!(builtin_publisher
            .datawriter_factory_lookup_datawriter(&subscription_topic)
            .is_ok());
        assert!(builtin_publisher
            .datawriter_factory_lookup_datawriter(&topic_topic)
            .is_ok());
    }
}
