use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
    sync::Mutex,
};

use rust_dds_api::{
    dcps_psm::{DomainId, StatusMask},
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::qos::{
        DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
        PublisherQos, SubscriberQos,
    },
    return_type::{DDSError, DDSResult},
};
use rust_dds_rtps_implementation::{
    data_representation_builtin_endpoints::{
        sedp_discovered_reader_data::{SedpDiscoveredReaderData, DCPS_SUBSCRIPTION},
        sedp_discovered_topic_data::{SedpDiscoveredTopicData, DCPS_TOPIC},
        sedp_discovered_writer_data::{SedpDiscoveredWriterData, DCPS_PUBLICATION},
        spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
    },
    dds_impl::{
        data_reader_proxy::{DataReaderAttributes, RtpsReader},
        data_writer_proxy::{DataWriterAttributes, RtpsWriter},
        domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
        publisher_proxy::PublisherAttributes,
        subscriber_proxy::SubscriberAttributes,
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
    behavior::writer::reader_locator::RtpsReaderLocatorConstructor,
    discovery::{
        sedp::builtin_endpoints::{
            SedpBuiltinPublicationsReader, SedpBuiltinPublicationsWriter,
            SedpBuiltinSubscriptionsReader, SedpBuiltinSubscriptionsWriter,
            SedpBuiltinTopicsReader, SedpBuiltinTopicsWriter,
        },
        spdp::builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
    },
    structure::{
        entity::RtpsEntityAttributes,
        group::RtpsGroupConstructor,
        types::{
            EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, BUILT_IN_READER_GROUP,
            BUILT_IN_WRITER_GROUP, PROTOCOLVERSION, VENDOR_ID_S2E,
        },
    },
};
use socket2::Socket;

use crate::{
    communication::Communication,
    tasks::{
        task_announce_participant, task_sedp_reader_discovery, task_sedp_writer_discovery,
        task_spdp_discovery, Executor, Spawner,
    },
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

// Note: the unicast address need to be configurable by the user later, and
// must also be retrieved dynamically (e.g. the IPv4 from the first network interface)
const UNICAST_LOCATOR_ADDRESS: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1];

// As of 9.6.1.4.1  Default multicast address
const DEFAULT_MULTICAST_LOCATOR_ADDRESS: [u8; 16] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1];

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

pub fn port_builtin_multicast(domain_id: u16) -> u16 {
    PB + DG * domain_id + d0
}

pub fn port_builtin_unicast(domain_id: u16, participant_id: u16) -> u16 {
    PB + DG * domain_id + d1 + PG * participant_id
}

pub fn port_user_unicast(domain_id: u16, participant_id: u16) -> u16 {
    PB + DG * domain_id + d3 + PG * participant_id
}

pub fn get_multicast_socket(
    address: Ipv4Addr,
    multicast_address: Ipv4Addr,
    port: u16,
) -> DDSResult<UdpSocket> {
    let socket_addr = SocketAddr::from((address, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;

    socket
        .set_reuse_address(true)
        .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;

    //socket.set_nonblocking(true).ok()?;
    socket
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;

    socket
        .bind(&socket_addr.into())
        .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;

    socket
        .join_multicast_v4(&multicast_address, &address)
        .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;
    socket
        .set_multicast_loop_v4(true)
        .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;

    Ok(socket.into())
}

pub fn get_unicast_socket<A: ToSocketAddrs>(addr: A) -> DDSResult<UdpSocket> {
    let socket =
        UdpSocket::bind(addr).map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;
    socket
        .set_nonblocking(true)
        .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))?;

    Ok(socket.into())
}

pub fn get_multicast_communication(
    guid_prefix: GuidPrefix,
    unicast_locator_list: &Vec<Locator>,
    multicast_locator_list: &Vec<Locator>,
) -> DDSResult<Communication<UdpTransport>> {
    let unicast_locator = unicast_locator_list
        .first()
        .ok_or(DDSError::PreconditionNotMet(
            "The unicast locator list is empty".to_string(),
        ))?;
    let multicast_locator = multicast_locator_list
        .first()
        .ok_or(DDSError::PreconditionNotMet(
            "The multicast locator list is empty".to_string(),
        ))?;

    let address = [
        unicast_locator.address[12],
        unicast_locator.address[13],
        unicast_locator.address[14],
        unicast_locator.address[15],
    ];
    let multicast_address = [
        multicast_locator.address[12],
        multicast_locator.address[13],
        multicast_locator.address[14],
        multicast_locator.address[15],
    ];

    Ok(Communication {
        version: PROTOCOLVERSION,
        vendor_id: VENDOR_ID_S2E,
        guid_prefix,
        transport: UdpTransport::new(get_multicast_socket(
            address.into(),
            multicast_address.into(),
            multicast_locator.port as u16,
        )?),
    })
}

pub fn get_unicast_communication(
    guid_prefix: GuidPrefix,
    unicast_locator_list: &Vec<Locator>,
) -> DDSResult<Communication<UdpTransport>> {
    let unicast_addresses = unicast_locator_list
        .iter()
        .map(|locator| {
            SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(
                    locator.address[12],
                    locator.address[13],
                    locator.address[14],
                    locator.address[15],
                )),
                locator.port as u16,
            )
        })
        .collect::<Vec<_>>();

    Ok(Communication {
        version: PROTOCOLVERSION,
        vendor_id: VENDOR_ID_S2E,
        guid_prefix,
        transport: UdpTransport::new(get_unicast_socket(unicast_addresses.as_slice())?),
    })
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
        let participant_id = self.participant_list.lock().unwrap().len();
        let guid_prefix = GuidPrefix([3; 12]);
        let qos = qos.unwrap_or_default();

        let domain_participant = RtpsShared::new(DomainParticipantAttributes::new(
            guid_prefix,
            domain_id,
            participant_id,
            "".to_string(),
            qos.clone(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, participant_id as u16) as u32,
                UNICAST_LOCATOR_ADDRESS,
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, participant_id as u16) as u32,
                UNICAST_LOCATOR_ADDRESS,
            )],
            vec![],
        ));

        create_builtins(domain_participant.clone())?;

        if qos.entity_factory.autoenable_created_entities {
            self.enable(domain_participant.clone())?;
        }

        self.participant_list
            .lock()
            .unwrap()
            .push(domain_participant.clone());

        Ok(DomainParticipantProxy::new(domain_participant.downgrade()))
    }

    fn enable(
        &self,
        domain_participant: RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
    ) -> DDSResult<()> {
        let guid_prefix = domain_participant
            .read_lock()
            .rtps_participant
            .guid()
            .prefix;

        // ////////// Task creation
        let (executor, spawner) = {
            let (sender, receiver) = std::sync::mpsc::sync_channel(10);
            (Executor { receiver }, Spawner::new(sender))
        };

        let mut metatraffic_multicast_communication = get_multicast_communication(
            guid_prefix,
            &domain_participant
                .read_lock()
                .metatraffic_unicast_locator_list,
            &domain_participant
                .read_lock()
                .metatraffic_multicast_locator_list,
        )?;
        let mut metatraffic_unicast_communication = get_unicast_communication(
            guid_prefix,
            &domain_participant
                .read_lock()
                .metatraffic_unicast_locator_list,
        )?;
        let mut default_unicast_communication = get_unicast_communication(
            guid_prefix,
            &domain_participant
                .read_lock()
                .rtps_participant
                .default_unicast_locator_list,
        )?;

        // //////////// SPDP Communication

        // ////////////// SPDP participant discovery
        {
            let domain_participant = domain_participant.clone();
            spawner.spawn_enabled_periodic_task(
                "builtin multicast communication",
                move || {
                    if let Some(builtin_participant_subscriber) =
                        &domain_participant.read_lock().builtin_subscriber
                    {
                        metatraffic_multicast_communication
                            .receive(core::slice::from_ref(builtin_participant_subscriber));
                    } else {
                        println!("/!\\ Participant has no builtin subscriber");
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        // ////////////// SPDP builtin endpoint configuration
        {
            let domain_participant = domain_participant.clone();

            spawner.spawn_enabled_periodic_task(
                "spdp endpoint configuration",
                move || match task_spdp_discovery(domain_participant.clone()) {
                    Ok(()) => (),
                    Err(e) => println!("spdp discovery failed: {:?}", e),
                },
                std::time::Duration::from_millis(500),
            );
        }

        // //////////// Unicast Communication
        {
            let domain_participant = domain_participant.clone();
            spawner.spawn_enabled_periodic_task(
                "builtin unicast communication",
                move || {
                    if let Some(builtin_publisher) =
                        &domain_participant.read_lock().builtin_publisher
                    {
                        metatraffic_unicast_communication
                            .send(core::slice::from_ref(builtin_publisher));
                    } else {
                        println!("/!\\ Participant has no builtin publisher");
                    }

                    if let Some(builtin_subscriber) =
                        &domain_participant.read_lock().builtin_subscriber
                    {
                        metatraffic_unicast_communication
                            .receive(core::slice::from_ref(builtin_subscriber));
                    } else {
                        println!("/!\\ Participant has no builtin subscriber");
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        // ////////////// SEDP user-defined endpoint configuration
        {
            let domain_participant = domain_participant.clone();

            spawner.spawn_enabled_periodic_task(
                "sedp user endpoint configuration",
                move || {
                    match task_sedp_writer_discovery(domain_participant.clone()) {
                        Ok(()) => (),
                        Err(e) => println!("sedp writer discovery failed: {:?}", e),
                    }
                    match task_sedp_reader_discovery(domain_participant.clone()) {
                        Ok(()) => (),
                        Err(e) => println!("sedp reader discovery failed: {:?}", e),
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        // //////////// User-defined Communication
        {
            let domain_participant = domain_participant.clone();
            spawner.spawn_enabled_periodic_task(
                "user-defined communication",
                move || {
                    default_unicast_communication.send(
                        domain_participant
                            .read_lock()
                            .user_defined_publisher_list
                            .as_ref(),
                    );

                    default_unicast_communication.receive(
                        domain_participant
                            .read_lock()
                            .user_defined_subscriber_list
                            .as_ref(),
                    );
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

        // //////////// Announce participant
        task_announce_participant(domain_participant)?;

        // //////////// Start running tasks
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

pub fn create_builtins(
    domain_participant: RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DDSResult<()> {
    let guid_prefix = domain_participant
        .read_lock()
        .rtps_participant
        .guid()
        .prefix;

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
            .push(spdp_builtin_participant_data_reader);

        let spdp_reader_locators: Vec<RtpsReaderLocatorAttributesImpl> = domain_participant
            .read_lock()
            .metatraffic_multicast_locator_list
            .iter()
            .map(|locator| RtpsReaderLocatorAttributesImpl::new(locator.clone(), false))
            .collect();

        let spdp_builtin_participant_rtps_writer = SpdpBuiltinParticipantWriter::create::<
            RtpsStatelessWriterImpl,
            _,
        >(
            guid_prefix, &[], &[], spdp_reader_locators
        );

        let spdp_builtin_participant_data_writer = RtpsShared::new(DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateless(spdp_builtin_participant_rtps_writer),
            spdp_topic_participant.clone(),
            builtin_publisher.downgrade(),
        ));
        builtin_publisher
            .write_lock()
            .data_writer_list
            .push(spdp_builtin_participant_data_writer);
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

    use mockall::mock;
    use rust_dds_api::{
        dcps_psm::{
            BuiltInTopicKey, DomainId, PublicationMatchedStatus, SubscriptionMatchedStatus, Time,
        },
        domain::domain_participant::{DomainParticipant, DomainParticipantTopicFactory},
        infrastructure::{entity::Entity, qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind},
        infrastructure::qos::DomainParticipantQos,
        publication::{
            data_writer::DataWriter,
            data_writer_listener::DataWriterListener,
            publisher::{Publisher, PublisherDataWriterFactory},
        },
        return_type::DDSError,
        subscription::{
            data_reader::DataReader,
            data_reader_listener::DataReaderListener,
            subscriber::{Subscriber, SubscriberDataReaderFactory},
        },
        topic::topic_description::TopicDescription,
    };
    use rust_dds_rtps_implementation::{
        data_representation_builtin_endpoints::{
            sedp_discovered_reader_data::SedpDiscoveredReaderData,
            sedp_discovered_topic_data::SedpDiscoveredTopicData,
            sedp_discovered_writer_data::SedpDiscoveredWriterData,
            spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
        },
        dds_impl::{
            domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
            publisher_proxy::PublisherProxy,
            subscriber_proxy::SubscriberProxy,
            topic_proxy::TopicProxy,
        },
        dds_type::{DdsDeserialize, DdsSerialize, DdsType},
        utils::shared_object::RtpsShared,
    };
    use rust_rtps_pim::structure::{
        entity::RtpsEntityAttributes,
        types::{GuidPrefix, LOCATOR_KIND_UDPv4, Locator},
    };

    use crate::{
        domain_participant_factory::{
            get_multicast_communication, get_multicast_socket, get_unicast_communication,
            port_builtin_multicast, port_user_unicast,
        },
        tasks::{
            task_announce_participant, task_sedp_reader_discovery, task_sedp_writer_discovery,
            task_spdp_discovery,
        },
    };

    use super::{
        create_builtins, port_builtin_unicast, RtpsStructureImpl, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    };

    #[rustfmt::skip]
    fn locator_address(address: &[u8; 4]) -> [u8; 16] {
        [
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            address[0], address[1], address[2], address[3]
        ]
    }

    #[test]
    fn multicast_socket_behaviour() {
        let port = 6000;
        let interface_addr = [127, 0, 0, 1];
        let multicast_ip = [239, 255, 0, 1];
        let multicast_addr = SocketAddr::from((multicast_ip, port));

        let socket1 =
            get_multicast_socket(interface_addr.into(), multicast_ip.into(), port).unwrap();
        let socket2 =
            get_multicast_socket(interface_addr.into(), multicast_ip.into(), port).unwrap();
        let socket3 =
            get_multicast_socket(interface_addr.into(), multicast_ip.into(), port).unwrap();

        socket1.send_to(&[1, 2, 3, 4], multicast_addr).unwrap();

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
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        create_builtins(domain_participant.clone()).unwrap();

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

    #[test]
    fn test_spdp_send_receive() {
        let domain_id = 4;
        let guid_prefix = GuidPrefix([3; 12]);
        let interface_address = [127, 0, 0, 1];
        let multicast_ip = [239, 255, 0, 1];

        // ////////// Create 2 participants
        let participant1 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            guid_prefix,
            domain_id,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 0) as u32,
                locator_address(&interface_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                locator_address(&multicast_ip),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 0) as u32,
                locator_address(&interface_address),
            )],
            vec![],
        ));
        create_builtins(participant1.clone()).unwrap();

        let participant2 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            guid_prefix,
            domain_id,
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 1) as u32,
                locator_address(&interface_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                locator_address(&multicast_ip),
            )],
            vec![],
            vec![],
        ));
        create_builtins(participant2.clone()).unwrap();

        // ////////// Send and receive SPDP data
        {
            task_announce_participant(participant1.clone()).unwrap();

            let mut communication_p1 = get_unicast_communication(
                guid_prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();

            let mut communication_p2 = get_multicast_communication(
                guid_prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
                &participant2.read_lock().metatraffic_multicast_locator_list,
            )
            .unwrap();

            communication_p1.send(core::slice::from_ref(
                participant1.read_lock().builtin_publisher.as_ref().unwrap(),
            ));

            communication_p2.receive(core::slice::from_ref(
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
        }

        // ////////// Participant 2 receives discovered participant data
        let spdp_discovered_participant_data = {
            let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());

            let subscriber = SubscriberProxy::new(
                participant2_proxy.clone(),
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap()
                    .downgrade(),
            );

            let participant_topic: TopicProxy<SpdpDiscoveredParticipantData, _> =
                participant2_proxy
                    .topic_factory_lookup_topicdescription(DCPS_PARTICIPANT)
                    .unwrap();
            let mut participant2_builtin_participant_data_reader = subscriber
                .datareader_factory_lookup_datareader(&participant_topic)
                .unwrap();

            &participant2_builtin_participant_data_reader
                .read(1, &[], &[], &[])
                .unwrap()[0]
        };

        // ////////// Check that the received data is correct
        {
            assert_eq!(
                BuiltInTopicKey {
                    value: participant1.read_lock().rtps_participant.guid().into()
                },
                spdp_discovered_participant_data.dds_participant_data.key,
            );

            assert_eq!(
                domain_id,
                spdp_discovered_participant_data.participant_proxy.domain_id as i32
            );

            assert_eq!(
                participant1.read_lock().rtps_participant.guid().prefix,
                spdp_discovered_participant_data
                    .participant_proxy
                    .guid_prefix
            );

            assert_eq!(
                participant1.read_lock().metatraffic_unicast_locator_list,
                spdp_discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
            );

            assert_eq!(
                participant1.read_lock().metatraffic_multicast_locator_list,
                spdp_discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
            );

            assert_eq!(
                participant1
                    .read_lock()
                    .rtps_participant
                    .default_unicast_locator_list,
                spdp_discovered_participant_data
                    .participant_proxy
                    .default_unicast_locator_list
            );
        }
    }

    struct UserData(u8);

    impl DdsType for UserData {
        fn type_name() -> &'static str {
            "UserData"
        }

        fn has_key() -> bool {
            false
        }
    }

    impl<'de> DdsDeserialize<'de> for UserData {
        fn deserialize(buf: &mut &'de [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
            Ok(UserData(buf[0]))
        }
    }

    impl DdsSerialize for UserData {
        fn serialize<W: std::io::Write, E: rust_dds_rtps_implementation::dds_type::Endianness>(
            &self,
            mut writer: W,
        ) -> rust_dds_api::return_type::DDSResult<()> {
            writer
                .write(&[self.0])
                .map(|_| ())
                .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))
        }
    }

    #[test]
    fn test_sedp_send_receive() {
        let domain_id = 5;
        let unicast_address = [127, 0, 0, 1];
        let multicast_address = [239, 255, 0, 1];

        // ////////// Create 2 participants
        let participant1 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            GuidPrefix([3; 12]),
            domain_id,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 0) as u32,
                locator_address(&unicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                locator_address(&multicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 0) as u32,
                locator_address(&unicast_address),
            )],
            vec![],
        ));
        let guid1 = participant1.read_lock().rtps_participant.guid().clone();
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());
        create_builtins(participant1.clone()).unwrap();

        let participant2 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            GuidPrefix([3; 12]),
            domain_id,
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 1) as u32,
                locator_address(&unicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                locator_address(&multicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 1) as u32,
                locator_address(&unicast_address),
            )],
            vec![],
        ));
        let guid2 = participant2.read_lock().rtps_participant.guid().clone();
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());
        create_builtins(participant2.clone()).unwrap();

        // Match SEDP endpoints
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            let mut communication_p1 = get_unicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_p2 = get_unicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_multicast_p1 = get_multicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
                &participant1.read_lock().metatraffic_multicast_locator_list,
            )
            .unwrap();
            let mut communication_multicast_p2 = get_multicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
                &participant2.read_lock().metatraffic_multicast_locator_list,
            )
            .unwrap();

            communication_p1.send(core::slice::from_ref(
                participant1.read_lock().builtin_publisher.as_ref().unwrap(),
            ));
            communication_p2.send(core::slice::from_ref(
                participant2.read_lock().builtin_publisher.as_ref().unwrap(),
            ));

            communication_multicast_p1.receive(core::slice::from_ref(
                participant1
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
            communication_multicast_p2.receive(core::slice::from_ref(
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));

            task_spdp_discovery(participant1.clone()).unwrap();
            task_spdp_discovery(participant2.clone()).unwrap();
        }

        // ////////// Create user endpoints
        let user_publisher = participant1_proxy.create_publisher(None, None, 0).unwrap();
        let user_subscriber = participant1_proxy.create_subscriber(None, None, 0).unwrap();

        let user_topic = participant1_proxy
            .create_topic::<UserData>("UserTopic", None, None, 0)
            .unwrap();
        let user_writer = user_publisher
            .create_datawriter(&user_topic, None, None, 0)
            .unwrap();
        let user_reader = user_subscriber
            .create_datareader(&user_topic, None, None, 0)
            .unwrap();

        // ////////// Send and receive SEDP data
        {
            let mut communication_p1 = get_unicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_p2 = get_unicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();

            communication_p1.send(core::slice::from_ref(
                participant1.read_lock().builtin_publisher.as_ref().unwrap(),
            ));
            communication_p2.send(core::slice::from_ref(
                participant2.read_lock().builtin_publisher.as_ref().unwrap(),
            ));

            communication_p1.receive(core::slice::from_ref(
                participant1
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
            communication_p2.receive(core::slice::from_ref(
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
        }

        // ////////// Check that the received data corresponds to the sent data

        let sedp_topic_publication: TopicProxy<SedpDiscoveredWriterData, _> = participant2_proxy
            .lookup_topicdescription(DCPS_PUBLICATION)
            .unwrap();
        let sedp_topic_subscription: TopicProxy<SedpDiscoveredReaderData, _> = participant2_proxy
            .lookup_topicdescription(DCPS_SUBSCRIPTION)
            .unwrap();
        let sedp_topic_topic: TopicProxy<SedpDiscoveredTopicData, _> = participant2_proxy
            .lookup_topicdescription(DCPS_TOPIC)
            .unwrap();

        let participant2_subscriber = SubscriberProxy::new(
            participant2_proxy,
            participant2
                .read_lock()
                .builtin_subscriber
                .as_ref()
                .unwrap()
                .downgrade(),
        );

        let mut participant2_publication_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_publication)
            .unwrap();
        let mut participant2_subscription_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_subscription)
            .unwrap();
        let mut participant2_topic_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_topic)
            .unwrap();

        let discovered_topic_data = &participant2_topic_datareader
            .read(1, &[], &[], &[])
            .unwrap()[0];
        assert_eq!(
            UserData::type_name(),
            discovered_topic_data.topic_builtin_topic_data.type_name,
        );
        assert_eq!(
            user_topic.get_name().unwrap(),
            discovered_topic_data.topic_builtin_topic_data.name,
        );

        let discovered_writer_data = &participant2_publication_datareader
            .read(1, &[], &[], &[])
            .unwrap()[0];
        assert_eq!(
            user_writer
                .as_ref()
                .upgrade()
                .unwrap()
                .write_lock()
                .rtps_writer
                .try_as_stateful_writer()
                .unwrap()
                .guid(),
            discovered_writer_data.writer_proxy.remote_writer_guid,
        );

        let discovered_reader_data = &participant2_subscription_datareader
            .read(1, &[], &[], &[])
            .unwrap()[0];
        assert_eq!(
            user_reader
                .as_ref()
                .upgrade()
                .unwrap()
                .write_lock()
                .rtps_reader
                .try_as_stateful_reader()
                .unwrap()
                .guid(),
            discovered_reader_data.reader_proxy.remote_reader_guid,
        );
    }

    mock! {
        #[derive(Clone)]
        ReaderListener {}

        impl DataReaderListener for ReaderListener {
            fn on_subscription_matched(&self, status: SubscriptionMatchedStatus);
            fn on_data_available(&self);
        }
    }

    mock! {
        #[derive(Clone)]
        WriterListener {}

        impl DataWriterListener for WriterListener {
            fn on_publication_matched(&self, status: PublicationMatchedStatus);
        }
    }

    #[test]
    fn test_reader_writer_matching_listener() {
        let domain_id = 6;
        let unicast_address = [127, 0, 0, 1];
        let multicast_address = [239, 255, 0, 1];

        // ////////// Create 2 participants
        let participant1 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            GuidPrefix([3; 12]),
            domain_id,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 0) as u32,
                locator_address(&unicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                locator_address(&multicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 0) as u32,
                locator_address(&unicast_address),
            )],
            vec![],
        ));
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());
        let guid1 = participant1.read_lock().rtps_participant.guid().clone();
        create_builtins(participant1.clone()).unwrap();

        let participant2 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            GuidPrefix([3; 12]),
            domain_id,
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 1) as u32,
                locator_address(&unicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                locator_address(&multicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 1) as u32,
                locator_address(&unicast_address),
            )],
            vec![],
        ));
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());
        let guid2 = participant2.read_lock().rtps_participant.guid().clone();
        create_builtins(participant2.clone()).unwrap();

        // ////////// Match SEDP endpoints
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            let mut communication_p1 = get_unicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_p2 = get_unicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_multicast_p1 = get_multicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
                &participant1.read_lock().metatraffic_multicast_locator_list,
            )
            .unwrap();
            let mut communication_multicast_p2 = get_multicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
                &participant2.read_lock().metatraffic_multicast_locator_list,
            )
            .unwrap();

            communication_p1.send(core::slice::from_ref(
                participant1.read_lock().builtin_publisher.as_ref().unwrap(),
            ));
            communication_p2.send(core::slice::from_ref(
                participant2.read_lock().builtin_publisher.as_ref().unwrap(),
            ));

            communication_multicast_p1.receive(core::slice::from_ref(
                participant1
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
            communication_multicast_p2.receive(core::slice::from_ref(
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));

            task_spdp_discovery(participant1.clone()).unwrap();
            task_spdp_discovery(participant2.clone()).unwrap();
        }

        // ////////// Write SEDP discovery data
        let user_publisher = participant1_proxy.create_publisher(None, None, 0).unwrap();
        let user_subscriber = participant2_proxy.create_subscriber(None, None, 0).unwrap();

        let user_topic = participant1_proxy
            .create_topic::<UserData>("UserTopic", None, None, 0)
            .unwrap();
        let user_writer = user_publisher
            .create_datawriter(
                &user_topic,
                None,
                Some(Box::new(MockWriterListener::new())),
                0,
            )
            .unwrap();
        let user_reader = user_subscriber
            .create_datareader(
                &user_topic,
                None,
                Some(Box::new(MockReaderListener::new())),
                0,
            )
            .unwrap();

        // ////////// Send SEDP data
        {
            let mut communication_p1 = get_unicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_p2 = get_unicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();

            communication_p1.send(core::slice::from_ref(
                participant1.read_lock().builtin_publisher.as_ref().unwrap(),
            ));
            communication_p2.send(core::slice::from_ref(
                participant2.read_lock().builtin_publisher.as_ref().unwrap(),
            ));

            communication_p1.receive(core::slice::from_ref(
                participant1
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
            communication_p2.receive(core::slice::from_ref(
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
        }

        // ////////// Process SEDP data

        // Writer listener must be called once on reader discovery
        {
            let mut writer_listener = Box::new(MockWriterListener::new());
            writer_listener
                .expect_on_publication_matched()
                .once()
                .return_const(());
            user_writer.set_listener(Some(writer_listener), 0).unwrap();

            task_sedp_reader_discovery(participant1.clone()).unwrap();

            user_writer
                .set_listener(Some(Box::new(MockWriterListener::new())), 0)
                .unwrap();
        }

        // Reader listener must be called once on writer discovery
        {
            let mut reader_listener = Box::new(MockReaderListener::new());
            reader_listener
                .expect_on_subscription_matched()
                .once()
                .return_const(());
            user_reader.set_listener(Some(reader_listener), 0).unwrap();

            task_sedp_writer_discovery(participant2.clone()).unwrap();

            user_reader
                .set_listener(Some(Box::new(MockReaderListener::new())), 0)
                .unwrap();
        }
    }

    #[test]
    fn test_reader_available_data_listener() {
        let domain_id = 7;
        let unicast_address = [127, 0, 0, 1];
        let multicast_address = [239, 255, 0, 1];

        // ////////// Create 2 participants
        let participant1 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            GuidPrefix([3; 12]),
            domain_id,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 0) as u32,
                locator_address(&unicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                locator_address(&multicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 0) as u32,
                locator_address(&unicast_address),
            )],
            vec![],
        ));
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());
        let guid1 = participant1.read_lock().rtps_participant.guid().clone();
        create_builtins(participant1.clone()).unwrap();

        let participant2 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            GuidPrefix([3; 12]),
            domain_id,
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 1) as u32,
                locator_address(&unicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                locator_address(&multicast_address),
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 1) as u32,
                locator_address(&unicast_address),
            )],
            vec![],
        ));
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());
        let guid2 = participant2.read_lock().rtps_participant.guid().clone();
        create_builtins(participant2.clone()).unwrap();

        // ////////// Match SEDP endpoints
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            let mut communication_p1 = get_unicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_p2 = get_unicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_multicast_p1 = get_multicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
                &participant1.read_lock().metatraffic_multicast_locator_list,
            )
            .unwrap();
            let mut communication_multicast_p2 = get_multicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
                &participant2.read_lock().metatraffic_multicast_locator_list,
            )
            .unwrap();

            communication_p1.send(core::slice::from_ref(
                participant1.read_lock().builtin_publisher.as_ref().unwrap(),
            ));
            communication_p2.send(core::slice::from_ref(
                participant2.read_lock().builtin_publisher.as_ref().unwrap(),
            ));

            communication_multicast_p1.receive(core::slice::from_ref(
                participant1
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
            communication_multicast_p2.receive(core::slice::from_ref(
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));

            task_spdp_discovery(participant1.clone()).unwrap();
            task_spdp_discovery(participant2.clone()).unwrap();
        }

        // ////////// Create user endpoints
        let user_publisher = participant1_proxy.create_publisher(None, None, 0).unwrap();
        let user_subscriber = participant2_proxy.create_subscriber(None, None, 0).unwrap();

        let user_topic = participant1_proxy
            .create_topic::<UserData>("UserTopic", None, None, 0)
            .unwrap();
        let mut user_writer = user_publisher
            .create_datawriter(&user_topic, None, None, 0)
            .unwrap();

        
        let mut reader_qos = DataReaderQos::default();
        reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;
        let user_reader = user_subscriber
            .create_datareader(
                &user_topic,
                Some(reader_qos),
                Some(Box::new(MockReaderListener::new())),
                0,
            )
            .unwrap();

        // ////////// Activate SEDP
        {
            let mut communication_p1 = get_unicast_communication(
                guid1.prefix,
                &participant1.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();
            let mut communication_p2 = get_unicast_communication(
                guid2.prefix,
                &participant2.read_lock().metatraffic_unicast_locator_list,
            )
            .unwrap();

            communication_p1.send(core::slice::from_ref(
                participant1.read_lock().builtin_publisher.as_ref().unwrap(),
            ));
            communication_p2.send(core::slice::from_ref(
                participant2.read_lock().builtin_publisher.as_ref().unwrap(),
            ));

            communication_p1.receive(core::slice::from_ref(
                participant1
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));
            communication_p2.receive(core::slice::from_ref(
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap(),
            ));

            // ////////// Process SEDP data
            task_sedp_reader_discovery(participant1.clone()).unwrap();

            // We expect the subscription matched listener to be called when matching
            let mut reader_listener = Box::new(MockReaderListener::new());
            reader_listener
                .expect_on_subscription_matched()
                .return_const(());
            user_reader.set_listener(Some(reader_listener), 0).unwrap();

            task_sedp_writer_discovery(participant2.clone()).unwrap();

            // No more listener should be called for now
            user_reader
                .set_listener(Some(Box::new(MockReaderListener::new())), 0)
                .unwrap();
        }

        // ////////// Write user data
        user_writer
            .write_w_timestamp(&UserData(8), None, Time { sec: 0, nanosec: 0 })
            .unwrap();

        // ////////// Send user data
        {
            let mut communication_p1 = get_unicast_communication(
                guid1.prefix,
                &participant1
                    .read_lock()
                    .rtps_participant
                    .default_unicast_locator_list,
            )
            .unwrap();

            let mut communication_p2 = get_unicast_communication(
                guid2.prefix,
                &participant2
                    .read_lock()
                    .rtps_participant
                    .default_unicast_locator_list,
            )
            .unwrap();

            communication_p1.send(&participant1.read_lock().user_defined_publisher_list);

            // On receive the available data listener should be called
            let mut reader_listener = Box::new(MockReaderListener::new());
            reader_listener
                .expect_on_data_available()
                .once()
                .return_const(());
            user_reader.set_listener(Some(reader_listener), 0).unwrap();

            communication_p2.receive(&participant2.read_lock().user_defined_subscriber_list);

            // From now on no listener should be called anymore
            user_reader
                .set_listener(Some(Box::new(MockReaderListener::new())), 0)
                .unwrap();
        }
    }
}
