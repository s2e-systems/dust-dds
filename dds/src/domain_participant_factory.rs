use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
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
    tasks::{
        task_sedp_reader_discovery, task_sedp_writer_discovery, task_spdp_discovery, Executor,
        Spawner,
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
) -> Option<UdpSocket> {
    let socket_addr = SocketAddr::from((address, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .ok()?;
    socket.set_reuse_address(true).ok()?;
    //socket.set_nonblocking(true).ok()?;
    socket
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .ok()?;
    socket.bind(&socket_addr.into()).ok()?;
    socket
        .join_multicast_v4(&multicast_address, &address)
        .ok()?;
    socket.set_multicast_loop_v4(true).ok()?;

    Some(socket.into())
}

pub fn get_unicast_socket(address: Ipv4Addr, port: u16) -> Option<UdpSocket> {
    let socket_addr = SocketAddr::from((address, port));

    let socket = UdpSocket::bind(socket_addr).ok()?;
    socket.set_nonblocking(true).ok()?;

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
        let domain_id = domain_participant.read_lock().domain_id;
        let domain_tag = domain_participant.read_lock().domain_tag.clone();
        let participant_id = self.participant_list.lock().unwrap().len();

        // ////////// Task creation
        let (executor, spawner) = {
            let (sender, receiver) = std::sync::mpsc::sync_channel(10);
            (Executor { receiver }, Spawner::new(sender))
        };

        // //////////// SPDP Communication

        // ////////////// SPDP participant discovery
        {
            let mut communication = Communication {
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix,
                transport: UdpTransport::new(
                    get_multicast_socket(
                        [
                            UNICAST_LOCATOR_ADDRESS[12],
                            UNICAST_LOCATOR_ADDRESS[13],
                            UNICAST_LOCATOR_ADDRESS[14],
                            UNICAST_LOCATOR_ADDRESS[15],
                        ]
                        .into(),
                        [
                            DEFAULT_MULTICAST_LOCATOR_ADDRESS[12],
                            DEFAULT_MULTICAST_LOCATOR_ADDRESS[13],
                            DEFAULT_MULTICAST_LOCATOR_ADDRESS[14],
                            DEFAULT_MULTICAST_LOCATOR_ADDRESS[15],
                        ]
                        .into(),
                        port_builtin_multicast(domain_id as u16),
                    )
                    .unwrap(),
                ),
            };

            let domain_participant = domain_participant.clone();
            spawner.spawn_enabled_periodic_task(
                "builtin multicast communication",
                move || {
                    if let Some(builtin_participant_subscriber) =
                        &domain_participant.read_lock().builtin_subscriber
                    {
                        communication
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
                    .lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
                    .ok()?;
                let publication_topic = participant_proxy
                    .lookup_topicdescription::<SedpDiscoveredWriterData>(DCPS_PUBLICATION)
                    .ok()?;
                let subscription_topic = participant_proxy
                    .lookup_topicdescription::<SedpDiscoveredReaderData>(DCPS_SUBSCRIPTION)
                    .ok()?;
                let topic_topic = participant_proxy
                    .lookup_topicdescription::<SedpDiscoveredTopicData>(DCPS_TOPIC)
                    .ok()?;

                let mut builtin_participant_data_reader = builtin_subscriber
                    .datareader_factory_lookup_datareader(&participant_topic)
                    .ok()?;

                let builtin_publication_reader = builtin_subscriber
                    .datareader_factory_lookup_datareader(&publication_topic)
                    .ok()?;
                let builtin_subscription_reader = builtin_subscriber
                    .datareader_factory_lookup_datareader(&subscription_topic)
                    .ok()?;
                let builtin_topic_reader = builtin_subscriber
                    .datareader_factory_lookup_datareader(&topic_topic)
                    .ok()?;
                let builtin_publication_writer = builtin_publisher
                    .datawriter_factory_lookup_datawriter(&publication_topic)
                    .ok()?;
                let builtin_subscription_writer = builtin_publisher
                    .datawriter_factory_lookup_datawriter(&subscription_topic)
                    .ok()?;
                let builtin_topic_writer = builtin_publisher
                    .datawriter_factory_lookup_datawriter(&topic_topic)
                    .ok()?;

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
                "spdp endpoint configuration",
                move || {
                    match try_perform_task() {
                        Some(()) => (),
                        None     => println!("spdp discovery failed (domain participant might not be fully constructed yet)")
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        // //////////// Unicast Communication
        {
            let mut communication = Communication {
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix,
                transport: UdpTransport::new(
                    get_unicast_socket(
                        [
                            UNICAST_LOCATOR_ADDRESS[12],
                            UNICAST_LOCATOR_ADDRESS[13],
                            UNICAST_LOCATOR_ADDRESS[14],
                            UNICAST_LOCATOR_ADDRESS[15],
                        ]
                        .into(),
                        port_builtin_unicast(domain_id as u16, participant_id as u16),
                    )
                    .unwrap(),
                ),
            };

            let domain_participant = domain_participant.clone();
            spawner.spawn_enabled_periodic_task(
                "builtin unicast communication",
                move || {
                    if let Some(builtin_publisher) =
                        &domain_participant.read_lock().builtin_publisher
                    {
                        communication.send(core::slice::from_ref(builtin_publisher));
                    } else {
                        println!("/!\\ Participant has no builtin publisher");
                    }

                    if let Some(builtin_subscriber) =
                        &domain_participant.read_lock().builtin_subscriber
                    {
                        communication.receive(core::slice::from_ref(builtin_subscriber));
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
                    .lookup_topicdescription::<SedpDiscoveredWriterData>(DCPS_PUBLICATION)
                    .ok()?;
                let mut builtin_publication_reader = builtin_subscriber
                    .datareader_factory_lookup_datareader(&publication_topic)
                    .ok()?;

                let subscription_topic = participant_proxy
                    .lookup_topicdescription::<SedpDiscoveredReaderData>(DCPS_SUBSCRIPTION)
                    .ok()?;
                let mut builtin_subscription_reader = builtin_subscriber
                    .datareader_factory_lookup_datareader(&subscription_topic)
                    .ok()?;

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
                "sedp user endpoint configuration",
                move || {
                    match try_perform_task() {
                        Some(()) => (),
                        None     => println!("sedp discovery failed (domain participant might not be fully constructed yet)")
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        // //////////// User-defined Communication
        {
            let mut communication = Communication {
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix,
                transport: UdpTransport::new(
                    get_unicast_socket(
                        [
                            UNICAST_LOCATOR_ADDRESS[12],
                            UNICAST_LOCATOR_ADDRESS[13],
                            UNICAST_LOCATOR_ADDRESS[14],
                            UNICAST_LOCATOR_ADDRESS[15],
                        ]
                        .into(),
                        port_user_unicast(domain_id as u16, participant_id as u16),
                    )
                    .unwrap(),
                ),
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

        let spdp_discovered_participant_data =
            spdp_discovered_participant_data_from_domain_participant(
                &domain_participant.read_lock(),
            );

        DataWriterProxy::new(builtin_participant_data_writer.downgrade()).write_w_timestamp(
            &spdp_discovered_participant_data,
            None,
            Time { sec: 0, nanosec: 0 },
        )?;

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

pub fn spdp_discovered_participant_data_from_domain_participant<Rtps>(
    participant: &DomainParticipantAttributes<Rtps>,
) -> SpdpDiscoveredParticipantData
where
    Rtps: RtpsStructure,
    Rtps::Participant: RtpsParticipantAttributes + RtpsEntityAttributes,
{
    SpdpDiscoveredParticipantData {
        dds_participant_data: ParticipantBuiltinTopicData {
            key: BuiltInTopicKey {
                value: (participant.rtps_participant.guid()).into(),
            },
            user_data: participant.qos.user_data.clone(),
        },
        participant_proxy: ParticipantProxy {
            domain_id: participant.domain_id as u32,
            domain_tag: participant.domain_tag.clone(),
            protocol_version: participant.rtps_participant.protocol_version(),
            guid_prefix: participant.rtps_participant.guid().prefix(),
            vendor_id: participant.rtps_participant.vendor_id(),
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

        let mut spdp_builtin_participant_rtps_writer =
            SpdpBuiltinParticipantWriter::create::<RtpsStatelessWriterImpl>(guid_prefix, &[], &[]);

        for locator in domain_participant
            .read_lock()
            .metatraffic_multicast_locator_list
            .iter()
        {
            spdp_builtin_participant_rtps_writer
                .reader_locator_add(RtpsReaderLocatorAttributesImpl::new(locator.clone(), false));
        }

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
        builtin_topics::{
            PublicationBuiltinTopicData, SubscriptionBuiltinTopicData, TopicBuiltinTopicData,
        },
        dcps_psm::{
            BuiltInTopicKey, DomainId, Duration, PublicationMatchedStatus,
            SubscriptionMatchedStatus, Time,
        },
        domain::domain_participant::{DomainParticipant, DomainParticipantTopicFactory},
        infrastructure::{
            qos::DomainParticipantQos,
            qos_policy::{
                DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
                DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy,
                LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy,
                OwnershipStrengthQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
                ReliabilityQosPolicy, ReliabilityQosPolicyKind, TimeBasedFilterQosPolicy,
                TopicDataQosPolicy, UserDataQosPolicy,
            },
        },
        publication::{
            data_writer::DataWriter,
            data_writer_listener::DataWriterListener,
            publisher::{Publisher, PublisherDataWriterFactory},
        },
        subscription::{
            data_reader::DataReader,
            data_reader_listener::DataReaderListener,
            subscriber::{Subscriber, SubscriberDataReaderFactory},
        },
        topic::topic_description::TopicDescription,
    };
    use rust_dds_rtps_implementation::{
        data_representation_builtin_endpoints::{
            sedp_discovered_reader_data::{RtpsReaderProxy, SedpDiscoveredReaderData},
            sedp_discovered_topic_data::SedpDiscoveredTopicData,
            sedp_discovered_writer_data::{RtpsWriterProxy, SedpDiscoveredWriterData},
            spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
        },
        dds_impl::{
            domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
            publisher_proxy::PublisherProxy,
            subscriber_proxy::SubscriberProxy,
            topic_proxy::TopicProxy,
        },
        dds_type::{DdsDeserialize, DdsSerialize, DdsType},
        rtps_impl::{
            rtps_reader_proxy_impl::RtpsReaderProxyAttributesImpl,
            rtps_writer_proxy_impl::RtpsWriterProxyImpl,
        },
        utils::shared_object::RtpsShared,
    };
    use rust_rtps_pim::{
        behavior::{
            reader::{
                stateful_reader::RtpsStatefulReaderOperations,
                writer_proxy::RtpsWriterProxyConstructor,
            },
            writer::{
                reader_proxy::RtpsReaderProxyConstructor,
                stateful_writer::RtpsStatefulWriterOperations,
            },
        },
        discovery::sedp::builtin_endpoints::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        },
        structure::{
            entity::RtpsEntityAttributes,
            participant::RtpsParticipantAttributes,
            types::{
                EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, ENTITYID_UNKNOWN,
                PROTOCOLVERSION, USER_DEFINED_READER_NO_KEY, USER_DEFINED_WRITER_NO_KEY,
                VENDOR_ID_S2E,
            },
        },
    };

    use crate::{
        communication::Communication,
        domain_participant_factory::{
            get_multicast_socket, port_builtin_multicast, port_user_unicast,
        },
        tasks::{task_sedp_reader_discovery, task_sedp_writer_discovery},
        udp_transport::UdpTransport,
    };

    use super::{
        create_builtins, get_unicast_socket, port_builtin_unicast,
        spdp_discovered_participant_data_from_domain_participant, RtpsStructureImpl,
        DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
    };

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
        let domain_id = 14;
        let guid_prefix = GuidPrefix([3; 12]);
        let interface_address = [127, 0, 0, 1];
        let interface_locator_address = [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            interface_address[0],
            interface_address[1],
            interface_address[2],
            interface_address[3],
        ];
        let multicast_ip = [239, 255, 0, 1];
        let multicast_locator_address = [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            multicast_ip[0],
            multicast_ip[1],
            multicast_ip[2],
            multicast_ip[3],
        ];

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
                interface_locator_address,
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                multicast_locator_address,
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 0) as u32,
                interface_locator_address,
            )],
            vec![],
        ));
        create_builtins(participant1.clone()).unwrap();
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());

        let participant2 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            guid_prefix,
            domain_id,
            1,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_unicast(domain_id as u16, 1) as u32,
                interface_locator_address,
            )],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_builtin_multicast(domain_id as u16) as u32,
                multicast_locator_address,
            )],
            vec![],
            vec![],
        ));
        create_builtins(participant2.clone()).unwrap();
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());

        let mut communication_p1 = Communication {
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: guid_prefix,
            transport: UdpTransport::new(
                get_unicast_socket(
                    interface_address.into(),
                    port_builtin_unicast(domain_id as u16, 0),
                )
                .unwrap(),
            ),
        };

        let mut communication_p2 = Communication {
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix,
            transport: UdpTransport::new(
                get_multicast_socket(
                    interface_address.into(),
                    multicast_ip.into(),
                    port_builtin_multicast(domain_id as u16),
                )
                .unwrap(),
            ),
        };

        // ////////// Participant 1 sends discovered participant data
        {
            let publisher = PublisherProxy::new(
                participant1
                    .read_lock()
                    .builtin_publisher
                    .as_ref()
                    .unwrap()
                    .downgrade(),
            );

            let mut participant1_builtin_participant_writer = {
                let participant_topic: TopicProxy<SpdpDiscoveredParticipantData, _> =
                    participant1_proxy
                        .topic_factory_lookup_topicdescription(DCPS_PARTICIPANT)
                        .unwrap();

                publisher
                    .datawriter_factory_lookup_datawriter(&participant_topic)
                    .unwrap()
            };

            participant1_builtin_participant_writer
                .write_w_timestamp(
                    &spdp_discovered_participant_data_from_domain_participant(
                        &participant1.read_lock(),
                    ),
                    None,
                    Time { sec: 0, nanosec: 0 },
                )
                .unwrap();

            communication_p1.send(&[publisher.as_ref().upgrade().unwrap()]);
        }

        // ////////// Participant 2 receives discovered participant data
        let spdp_discovered_participant_data = {
            let subscriber = SubscriberProxy::new(
                participant2_proxy.clone(),
                participant2
                    .read_lock()
                    .builtin_subscriber
                    .as_ref()
                    .unwrap()
                    .downgrade(),
            );

            communication_p2.receive(&[subscriber.as_ref().upgrade().unwrap()]);

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

        assert_eq!(
            spdp_discovered_participant_data,
            &spdp_discovered_participant_data_from_domain_participant(&participant1.read_lock()),
        );
    }

    struct UserData;

    impl DdsType for UserData {
        fn type_name() -> &'static str {
            "UserData"
        }

        fn has_key() -> bool {
            false
        }
    }

    impl<'de> DdsDeserialize<'de> for UserData {
        fn deserialize(_buf: &mut &'de [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
            Ok(UserData)
        }
    }

    impl DdsSerialize for UserData {
        fn serialize<W: std::io::Write, E: rust_dds_rtps_implementation::dds_type::Endianness>(
            &self,
            _writer: W,
        ) -> rust_dds_api::return_type::DDSResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_sedp_send_receive() {
        let domain_id = 10;
        let unicast_address = [127, 0, 0, 1];
        #[rustfmt::skip]
        let unicast_locator_address =
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, unicast_address[0], unicast_address[1], unicast_address[2], unicast_address[3]];

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
                unicast_locator_address,
            )],
            vec![],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 0) as u32,
                unicast_locator_address,
            )],
            vec![],
        ));
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
                unicast_locator_address,
            )],
            vec![],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 1) as u32,
                unicast_locator_address,
            )],
            vec![],
        ));
        let guid2 = participant2.read_lock().rtps_participant.guid().clone();
        create_builtins(participant2.clone()).unwrap();

        // ////////// Match builtin data readers/writers
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());
        let sedp_topic_publication: TopicProxy<SedpDiscoveredWriterData, _> = participant1_proxy
            .lookup_topicdescription(DCPS_PUBLICATION)
            .unwrap();
        let sedp_topic_subscription: TopicProxy<SedpDiscoveredReaderData, _> = participant1_proxy
            .lookup_topicdescription(DCPS_SUBSCRIPTION)
            .unwrap();
        let sedp_topic_topic: TopicProxy<SedpDiscoveredTopicData, _> = participant1_proxy
            .lookup_topicdescription(DCPS_TOPIC)
            .unwrap();

        let participant1_publisher = PublisherProxy::new(
            participant1
                .read_lock()
                .builtin_publisher
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant2_subscriber = SubscriberProxy::new(
            participant2_proxy,
            participant2
                .read_lock()
                .builtin_subscriber
                .as_ref()
                .unwrap()
                .downgrade(),
        );

        let mut participant1_publication_datawriter = participant1_publisher
            .lookup_datawriter(&sedp_topic_publication)
            .unwrap();
        let mut participant2_publication_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_publication)
            .unwrap();
        participant1_publication_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant2_publication_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        let mut participant1_subscription_datawriter = participant1_publisher
            .lookup_datawriter(&sedp_topic_subscription)
            .unwrap();
        let mut participant2_subscription_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_subscription)
            .unwrap();
        participant1_subscription_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant2_subscription_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        let mut participant1_topic_datawriter = participant1_publisher
            .lookup_datawriter(&sedp_topic_topic)
            .unwrap();
        let mut participant2_topic_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_topic)
            .unwrap();
        participant1_topic_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant2_topic_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        // ////////// Write SEDP discovery data
        let discovered_topic_data = SedpDiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey { value: [1; 16] },
                name: "UserTopic".to_string(),
                type_name: UserData::type_name().to_string(),
                durability: Default::default(),
                durability_service: Default::default(),
                deadline: Default::default(),
                latency_budget: Default::default(),
                liveliness: Default::default(),
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
                    max_blocking_time: Duration::new(0, 0),
                },
                transport_priority: Default::default(),
                lifespan: Default::default(),
                destination_order: Default::default(),
                history: Default::default(),
                resource_limits: Default::default(),
                ownership: Default::default(),
                topic_data: Default::default(),
            },
        };
        participant1_topic_datawriter
            .write_w_timestamp(&discovered_topic_data, None, Time { sec: 0, nanosec: 0 })
            .unwrap();

        let remote_writer_guid = Guid::new(
            guid1.prefix,
            EntityId::new([0, 0, 0], USER_DEFINED_WRITER_NO_KEY),
        );
        let discovered_writer_data = SedpDiscoveredWriterData {
            writer_proxy: RtpsWriterProxy {
                remote_writer_guid,
                unicast_locator_list: participant1
                    .read_lock()
                    .rtps_participant
                    .default_unicast_locator_list()
                    .to_vec(),
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId::new([0; 3], 0),
            },
            publication_builtin_topic_data: PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: remote_writer_guid.into(),
                },
                participant_key: BuiltInTopicKey { value: [1; 16] },
                topic_name: "UserTopic".to_string(),
                type_name: UserData::type_name().to_string(),
                durability: DurabilityQosPolicy::default(),
                durability_service: DurabilityServiceQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                    max_blocking_time: Duration::new(3, 0),
                },
                lifespan: LifespanQosPolicy::default(),
                user_data: UserDataQosPolicy::default(),
                ownership: OwnershipQosPolicy::default(),
                ownership_strength: OwnershipStrengthQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                presentation: PresentationQosPolicy::default(),
                partition: PartitionQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
                group_data: GroupDataQosPolicy::default(),
            },
        };
        participant1_publication_datawriter
            .write_w_timestamp(&discovered_writer_data, None, Time { sec: 0, nanosec: 0 })
            .unwrap();

        let remote_reader_guid = Guid::new(
            guid1.prefix,
            EntityId::new([0, 0, 0], USER_DEFINED_READER_NO_KEY),
        );
        let discovered_reader_data = SedpDiscoveredReaderData {
            reader_proxy: RtpsReaderProxy {
                remote_reader_guid,
                remote_group_entity_id: EntityId::new([1; 3], 0),
                unicast_locator_list: participant1
                    .read_lock()
                    .rtps_participant
                    .default_unicast_locator_list()
                    .to_vec(),
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },
            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: remote_reader_guid.into(),
                },
                participant_key: BuiltInTopicKey { value: [1; 16] },
                topic_name: "UserTopic".to_string(),
                type_name: UserData::type_name().to_string(),
                durability: DurabilityQosPolicy::default(),
                deadline: DeadlineQosPolicy::default(),
                latency_budget: LatencyBudgetQosPolicy::default(),
                liveliness: LivelinessQosPolicy::default(),
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                    max_blocking_time: Duration::new(3, 0),
                },
                ownership: OwnershipQosPolicy::default(),
                destination_order: DestinationOrderQosPolicy::default(),
                user_data: UserDataQosPolicy::default(),
                time_based_filter: TimeBasedFilterQosPolicy::default(),
                presentation: PresentationQosPolicy::default(),
                partition: PartitionQosPolicy::default(),
                topic_data: TopicDataQosPolicy::default(),
                group_data: GroupDataQosPolicy::default(),
            },
        };
        participant1_subscription_datawriter
            .write_w_timestamp(&discovered_reader_data, None, Time { sec: 0, nanosec: 0 })
            .unwrap();

        // ////////// Create communications for the 2 participants and send data from P1 to P2
        let mut communication_p1 = Communication {
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: guid1.prefix,
            transport: UdpTransport::new(
                get_unicast_socket(
                    unicast_address.into(),
                    port_builtin_unicast(domain_id as u16, 0),
                )
                .unwrap(),
            ),
        };

        let mut communication_p2 = Communication {
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: guid2.prefix,
            transport: UdpTransport::new(
                get_unicast_socket(
                    unicast_address.into(),
                    port_builtin_unicast(domain_id as u16, 1),
                )
                .unwrap(),
            ),
        };

        communication_p1.send(&[participant1_publisher.as_ref().upgrade().unwrap()]);
        communication_p2.receive(&[participant2_subscriber.as_ref().upgrade().unwrap()]);

        // ////////// Check that the received data corresponds to the sent data

        assert_eq!(
            discovered_topic_data,
            participant2_topic_datareader
                .read(1, &[], &[], &[])
                .unwrap()[0],
        );
        assert_eq!(
            discovered_writer_data,
            participant2_publication_datareader
                .read(1, &[], &[], &[])
                .unwrap()[0],
        );
        assert_eq!(
            discovered_reader_data,
            participant2_subscription_datareader
                .read(1, &[], &[], &[])
                .unwrap()[0],
        );
    }

    #[test]
    fn test_sedp_send_receive_with_factory() {
        let domain_id = 4;
        let unicast_address = [127, 0, 0, 1];
        #[rustfmt::skip]
        let unicast_locator_address =
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, unicast_address[0], unicast_address[1], unicast_address[2], unicast_address[3]];

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
                unicast_locator_address,
            )],
            vec![],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 0) as u32,
                unicast_locator_address,
            )],
            vec![],
        ));
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
                unicast_locator_address,
            )],
            vec![],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 1) as u32,
                unicast_locator_address,
            )],
            vec![],
        ));
        let guid2 = participant2.read_lock().rtps_participant.guid().clone();
        create_builtins(participant2.clone()).unwrap();

        // ////////// Match builtin data readers/writers
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());
        let sedp_topic_publication: TopicProxy<SedpDiscoveredWriterData, _> = participant1_proxy
            .lookup_topicdescription(DCPS_PUBLICATION)
            .unwrap();
        let sedp_topic_subscription: TopicProxy<SedpDiscoveredReaderData, _> = participant1_proxy
            .lookup_topicdescription(DCPS_SUBSCRIPTION)
            .unwrap();
        let sedp_topic_topic: TopicProxy<SedpDiscoveredTopicData, _> = participant1_proxy
            .lookup_topicdescription(DCPS_TOPIC)
            .unwrap();

        let participant1_publisher = PublisherProxy::new(
            participant1
                .read_lock()
                .builtin_publisher
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant2_subscriber = SubscriberProxy::new(
            participant2_proxy,
            participant2
                .read_lock()
                .builtin_subscriber
                .as_ref()
                .unwrap()
                .downgrade(),
        );

        let participant1_publication_datawriter = participant1_publisher
            .lookup_datawriter(&sedp_topic_publication)
            .unwrap();
        let mut participant2_publication_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_publication)
            .unwrap();
        participant1_publication_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant2_publication_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        let participant1_subscription_datawriter = participant1_publisher
            .lookup_datawriter(&sedp_topic_subscription)
            .unwrap();
        let mut participant2_subscription_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_subscription)
            .unwrap();
        participant1_subscription_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant2_subscription_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        let participant1_topic_datawriter = participant1_publisher
            .lookup_datawriter(&sedp_topic_topic)
            .unwrap();
        let mut participant2_topic_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_topic)
            .unwrap();
        participant1_topic_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant2_topic_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        // ////////// Write SEDP discovery data
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

        // ////////// Create communications for the 2 participants and send data from P1 to P2
        let mut communication_p1 = Communication {
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: guid1.prefix,
            transport: UdpTransport::new(
                get_unicast_socket(
                    unicast_address.into(),
                    port_builtin_unicast(domain_id as u16, 0),
                )
                .unwrap(),
            ),
        };

        let mut communication_p2 = Communication {
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: guid2.prefix,
            transport: UdpTransport::new(
                get_unicast_socket(
                    unicast_address.into(),
                    port_builtin_unicast(domain_id as u16, 1),
                )
                .unwrap(),
            ),
        };

        communication_p1.send(&[participant1_publisher.as_ref().upgrade().unwrap()]);
        communication_p2.receive(&[participant2_subscriber.as_ref().upgrade().unwrap()]);

        // ////////// Check that the received data corresponds to the sent data

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
        ReaderListener {}

        impl DataReaderListener for ReaderListener {
            fn on_subscription_matched(&self, status: SubscriptionMatchedStatus);
        }
    }

    mock! {
        WriterListener {}

        impl DataWriterListener for WriterListener {
            fn on_publication_matched(&self, status: PublicationMatchedStatus);
        }
    }

    #[test]
    fn test_reader_writer_listener() {
        let domain_id = 8;
        let unicast_address = [127, 0, 0, 1];
        #[rustfmt::skip]
        let unicast_locator_address =
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, unicast_address[0], unicast_address[1], unicast_address[2], unicast_address[3]];

        // ////////// Create the listeners
        let mut writer_listener = MockWriterListener::new();
        writer_listener.expect_on_publication_matched().times(1).return_const(());

        let mut reader_listener = MockReaderListener::new();
        reader_listener.expect_on_subscription_matched().times(1).return_const(());

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
                unicast_locator_address,
            )],
            vec![],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 0) as u32,
                unicast_locator_address,
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
                unicast_locator_address,
            )],
            vec![],
            vec![Locator::new(
                LOCATOR_KIND_UDPv4,
                port_user_unicast(domain_id as u16, 1) as u32,
                unicast_locator_address,
            )],
            vec![],
        ));
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());
        let guid2 = participant2.read_lock().rtps_participant.guid().clone();
        create_builtins(participant2.clone()).unwrap();

        // ////////// Match builtin data readers/writers
        let sedp_topic_publication: TopicProxy<SedpDiscoveredWriterData, _> = participant1_proxy
            .lookup_topicdescription(DCPS_PUBLICATION)
            .unwrap();
        let sedp_topic_subscription: TopicProxy<SedpDiscoveredReaderData, _> = participant1_proxy
            .lookup_topicdescription(DCPS_SUBSCRIPTION)
            .unwrap();

        let participant1_publisher = PublisherProxy::new(
            participant1
                .read_lock()
                .builtin_publisher
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant1_subscriber = SubscriberProxy::new(
            participant1_proxy.clone(),
            participant1
                .read_lock()
                .builtin_subscriber
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant2_publisher = PublisherProxy::new(
            participant2
                .read_lock()
                .builtin_publisher
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant2_subscriber = SubscriberProxy::new(
            participant2_proxy.clone(),
            participant2
                .read_lock()
                .builtin_subscriber
                .as_ref()
                .unwrap()
                .downgrade(),
        );

        let participant1_publication_datawriter = participant1_publisher
            .lookup_datawriter(&sedp_topic_publication)
            .unwrap();
        let mut participant2_publication_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_publication)
            .unwrap();
        participant1_publication_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant2_publication_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));


        let participant2_publication_datawriter = participant2_publisher
            .lookup_datawriter(&sedp_topic_publication)
            .unwrap();
        let participant1_publication_datareader = participant1_subscriber
            .lookup_datareader(&sedp_topic_publication)
            .unwrap();
        participant2_publication_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant1_publication_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        let participant1_subscription_datawriter = participant1_publisher
            .lookup_datawriter(&sedp_topic_subscription)
            .unwrap();
        let participant2_subscription_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_subscription)
            .unwrap();
        participant1_subscription_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant2_subscription_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        let participant2_subscription_datawriter = participant2_publisher
            .lookup_datawriter(&sedp_topic_subscription)
            .unwrap();
        let mut participant1_subscription_datareader = participant1_subscriber
            .lookup_datareader(&sedp_topic_subscription)
            .unwrap();
        participant2_subscription_datawriter
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_writer
            .try_as_stateful_writer()
            .unwrap()
            .matched_reader_add(RtpsReaderProxyAttributesImpl::new(
                Guid::new(guid1.prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
                ENTITYID_UNKNOWN,
                participant1
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                false,
                true,
            ));
        participant1_subscription_datareader
            .as_ref()
            .upgrade()
            .unwrap()
            .write_lock()
            .rtps_reader
            .try_as_stateful_reader()
            .unwrap()
            .matched_writer_add(RtpsWriterProxyImpl::new(
                Guid::new(guid2.prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
                participant2
                    .read_lock()
                    .metatraffic_unicast_locator_list
                    .as_slice(),
                &[],
                None,
                ENTITYID_UNKNOWN,
            ));

        // ////////// Write SEDP discovery data
        let user_publisher = participant1_proxy.create_publisher(None, None, 0).unwrap();
        let user_subscriber = participant2_proxy.create_subscriber(None, None, 0).unwrap();

        let user_topic = participant1_proxy
            .create_topic::<UserData>("UserTopic", None, None, 0)
            .unwrap();
        user_publisher
            .create_datawriter(&user_topic, None, Some(Box::new(writer_listener)), 0)
            .unwrap();
        user_subscriber
            .create_datareader(&user_topic, None, Some(Box::new(reader_listener)), 0)
            .unwrap();

        // ////////// Create communications for the 2 participants and send data from P1 to P2
        let mut communication_p1 = Communication {
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: guid1.prefix,
            transport: UdpTransport::new(
                get_unicast_socket(
                    unicast_address.into(),
                    port_builtin_unicast(domain_id as u16, 0),
                )
                .unwrap(),
            ),
        };

        let mut communication_p2 = Communication {
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: guid2.prefix,
            transport: UdpTransport::new(
                get_unicast_socket(
                    unicast_address.into(),
                    port_builtin_unicast(domain_id as u16, 1),
                )
                .unwrap(),
            ),
        };

        communication_p1.send(&[participant1_publisher.as_ref().upgrade().unwrap()]);
        communication_p2.send(&[participant2_publisher.as_ref().upgrade().unwrap()]);
        communication_p1.receive(&[participant1_subscriber.as_ref().upgrade().unwrap()]);
        communication_p2.receive(&[participant2_subscriber.as_ref().upgrade().unwrap()]);

        // ////////// call sedp task

        task_sedp_reader_discovery(
            &mut participant1_subscription_datareader,
            &participant1.read_lock().user_defined_publisher_list,
        );
        task_sedp_writer_discovery(
            &mut participant2_publication_datareader,
            &participant2.read_lock().user_defined_subscriber_list,
        );
    }
}
