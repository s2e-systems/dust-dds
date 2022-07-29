use std::sync::Mutex;

use crate::{
    dcps_psm::{DomainId, StatusMask},
    implementation::rtps::types::GuidPrefix,
    infrastructure::{
        entity::Entity,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos},
    },
};
use crate::{
    implementation::{
        dds_impl::domain_participant_impl::{
            AnnounceParticipant, CreateBuiltIns, DomainParticipantImpl, ReceiveBuiltInData,
            ReceiveUserDefinedData, SedpReaderDiscovery, SedpWriterDiscovery, SendBuiltInData,
            SendUserDefinedData, SpdpParticipantDiscovery,
        },
        task_manager::TaskManager,
        utils::shared_object::DdsShared,
    },
    return_type::{DdsError, DdsResult},
};

use lazy_static::lazy_static;
use rtps_udp_psm::udp_transport::RtpsUdpPsm;

use crate::domain::domain_participant_listener::DomainParticipantListener;

use super::domain_participant::DomainParticipant;

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

struct ParticipantManager {
    participant: DdsShared<DomainParticipantImpl>,
    task_manager: TaskManager,
}

impl ParticipantManager {
    fn start(&mut self, mut transport: RtpsUdpPsm) {
        //     let mut metatraffic_multicast_communication = communications.metatraffic_multicast;
        //     let mut metatraffic_unicast_communication = communications.metatraffic_unicast;
        //     let mut default_unicast_communication = communications.default_unicast;

        // //////////// SPDP Communication

        // ////////////// SPDP participant discovery
        {
            let domain_participant = self.participant.clone();
            let mut metatraffic_multicast_transport =
                transport.metatraffic_multicast_transport().unwrap();
            self.task_manager.spawn_enabled_periodic_task(
                "builtin multicast communication",
                move || {
                    domain_participant.receive_built_in_data(&mut metatraffic_multicast_transport);
                },
                std::time::Duration::from_millis(500),
            );
        }

        // ////////////// SPDP builtin endpoint configuration
        {
            let domain_participant = self.participant.clone();

            self.task_manager.spawn_enabled_periodic_task(
                "spdp endpoint configuration",
                move || match domain_participant.discover_matched_participants() {
                    Ok(()) => (),
                    Err(e) => println!("spdp discovery failed: {:?}", e),
                },
                std::time::Duration::from_millis(500),
            );
        }

        // //////////// Unicast Communication
        {
            let domain_participant = self.participant.clone();
            let mut metatraffic_unicast_transport =
                transport.metatraffic_unicast_transport().unwrap();
            self.task_manager.spawn_enabled_periodic_task(
                "builtin unicast communication",
                move || {
                    domain_participant.send_built_in_data(&mut metatraffic_unicast_transport);
                    domain_participant.receive_built_in_data(&mut metatraffic_unicast_transport);
                },
                std::time::Duration::from_millis(500),
            );
        }

        // ////////////// SEDP user-defined endpoint configuration
        {
            let domain_participant = self.participant.clone();

            self.task_manager.spawn_enabled_periodic_task(
                "sedp user endpoint configuration",
                move || {
                    match domain_participant.discover_matched_writers() {
                        Ok(()) => (),
                        Err(e) => println!("sedp writer discovery failed: {:?}", e),
                    }
                    match domain_participant.discover_matched_readers() {
                        Ok(()) => (),
                        Err(e) => println!("sedp reader discovery failed: {:?}", e),
                    }
                },
                std::time::Duration::from_millis(500),
            );
        }

        // //////////// User-defined Communication
        {
            let domain_participant = self.participant.clone();
            let mut default_unicast_transport = transport.default_unicast_transport().unwrap();
            self.task_manager.spawn_enabled_periodic_task(
                "user-defined communication",
                move || {
                    domain_participant.send_user_defined_data(&mut default_unicast_transport);
                    domain_participant.receive_user_defined_data(&mut default_unicast_transport);
                },
                std::time::Duration::from_millis(50),
            );
        }

        // //////////// Announce participant
        let domain_participant = self.participant.clone();
        self.task_manager.spawn_enabled_periodic_task(
            "participant announcement",
            move || match domain_participant.announce_participant() {
                Ok(_) => (),
                Err(e) => println!("participant announcement failed: {:?}", e),
            },
            std::time::Duration::from_millis(5000),
        );

        // //////////// Start running tasks
        self.task_manager.enable_tasks();
    }
}

lazy_static! {
    static ref THE_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory {
        participant_list: Mutex::new(vec![])
    };
}

pub struct DomainParticipantFactory {
    participant_list: Mutex<Vec<ParticipantManager>>,
}

pub trait DdsDomainParticipantFactory<T = RtpsUdpPsm> {
    /// This operation creates a new DomainParticipant object. The DomainParticipant signifies that the calling application intends
    /// to join the Domain identified by the domain_id argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no DomainParticipant will be created.
    /// The special value PARTICIPANT_QOS_DEFAULT can be used to indicate that the DomainParticipant should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation get_default_participant_qos (2.2.2.2.2.6) and using the resulting
    /// QoS to create the DomainParticipant.
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    fn create_participant(
        &self,
        domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: StatusMask,
    ) -> DdsResult<DomainParticipant>;
}

impl DdsDomainParticipantFactory for DomainParticipantFactory {
    fn create_participant(
        &self,
        domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: StatusMask,
    ) -> DdsResult<DomainParticipant> {
        let qos = qos.unwrap_or_default();

        let rtps_udp_psm = RtpsUdpPsm::new(domain_id).map_err(DdsError::PreconditionNotMet)?;

        let domain_participant = DomainParticipantImpl::new(
            GuidPrefix(rtps_udp_psm.guid_prefix()),
            domain_id,
            "".to_string(),
            qos.clone(),
            rtps_udp_psm.metatraffic_unicast_locator_list(),
            rtps_udp_psm.metatraffic_multicast_locator_list(),
            rtps_udp_psm.default_unicast_locator_list(),
            vec![],
        );

        if qos.entity_factory.autoenable_created_entities {
            domain_participant.enable()?;
        }

        domain_participant.create_builtins()?;

        if qos.entity_factory.autoenable_created_entities {
            domain_participant.enable()?;
        }

        let mut participant_manager = ParticipantManager {
            participant: domain_participant,
            task_manager: TaskManager::new(),
        };
        participant_manager.start(rtps_udp_psm);

        let participant_proxy = DomainParticipant::new(participant_manager.participant.downgrade());

        THE_PARTICIPANT_FACTORY
            .participant_list
            .lock()
            .unwrap()
            .push(participant_manager);

        Ok(participant_proxy)
    }
}

impl DomainParticipantFactory {
    /// This operation deletes an existing DomainParticipant. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted. Otherwise the error PRECONDITION_NOT_MET is returned.
    /// Possible error codes returned in addition to the standard ones: PRECONDITION_NOT_MET.
    pub fn delete_participant(&self, participant: &DomainParticipant) -> DdsResult<()> {
        let mut participant_list = THE_PARTICIPANT_FACTORY
            .participant_list
            .lock()
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;

        let index = participant_list
            .iter()
            .position(|pm| DomainParticipant::new(pm.participant.downgrade()).eq(participant))
            .ok_or(DdsError::AlreadyDeleted)?;

        participant_list.remove(index);

        Ok(())
    }

    /// This operation returns the DomainParticipantFactory singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same DomainParticipantFactory instance.
    /// The get_instance operation is a static operation implemented using the syntax of the native language and can therefore not be
    /// expressed in the IDL PSM.
    /// The pre-defined value TheParticipantFactory can also be used as an alias for the singleton factory returned by the operation
    /// get_instance.
    pub fn get_instance() -> &'static Self {
        &THE_PARTICIPANT_FACTORY
    }

    /// This operation retrieves a previously created DomainParticipant belonging to specified domain_id. If no such
    /// DomainParticipant exists, the operation will return a ‘nil’ value.
    /// If multiple DomainParticipant entities belonging to that domain_id exist, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_participant(&self, _domain_id: DomainId) -> DdsResult<DomainParticipant> {
        todo!()
    }

    /// This operation sets a default value of the DomainParticipant QoS policies which will be used for newly created
    /// DomainParticipant entities in the case where the QoS policies are defaulted in the create_participant operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    pub fn set_default_participant_qos(&self, _qos: Option<DomainParticipantQos>) {
        todo!()
    }

    /// This operation retrieves the default value of the DomainParticipant QoS, that is, the QoS policies which will be used for
    /// newly created DomainParticipant entities in the case where the QoS policies are defaulted in the create_participant
    /// operation.
    /// The values retrieved get_default_participant_qos will match the set of values specified on the last successful call to
    /// set_default_participant_qos, or else, if the call was never made, the default values listed in the QoS table in 2.2.3,
    /// Supported QoS.
    pub fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        todo!()
    }

    /// This operation sets the value of the DomainParticipantFactory QoS policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the DomainParticipantFactory is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return INCONSISTENT_POLICY.
    pub fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        todo!()
    }

    /// This operation returns the value of the DomainParticipantFactory QoS policies.
    pub fn set_qos(&self, _qos: Option<DomainParticipantFactoryQos>) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        implementation::rtps::types::{Guid, ENTITYID_PARTICIPANT, GUIDPREFIX_UNKNOWN},
        publication::data_writer::DataWriterProxy,
        publication::data_writer_listener::DataWriterListener,
        subscription::{
            data_reader::DataReader, data_reader_listener::DataReaderListener,
            subscriber::Subscriber,
        },
        topic_definition::topic::Topic,
    };

    use super::*;
    use crate::{
        dcps_psm::{
            BuiltInTopicKey, PublicationMatchedStatus, SubscriptionMatchedStatus,
            ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE,
        },
        infrastructure::{
            entity::Entity, qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind,
        },
    };
    use crate::{
        dds_type::{DdsDeserialize, DdsSerialize, DdsType},
        implementation::{
            data_representation_builtin_endpoints::{
                discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
                discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
                discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
                spdp_discovered_participant_data::{
                    SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
                },
            },
            dds_impl::domain_participant_impl::CreateBuiltIns,
        },
    };
    use dds_transport::{TransportRead, TransportWrite};
    use mockall::mock;
    use rtps_pim::structure::types::{LOCATOR_KIND_UDPv4, Locator};
    use rtps_udp_psm::mapping_traits::{from_bytes, to_bytes};

    struct Mailbox {
        received_messages: Vec<(Locator, Vec<u8>)>,
        read: usize,
    }

    impl Mailbox {
        fn new() -> Self {
            Self {
                received_messages: vec![],
                read: 0,
            }
        }
    }

    impl TransportWrite for Mailbox {
        fn write(
            &mut self,
            message: &dds_transport::RtpsMessage<'_>,
            destination_locator: rtps_pim::structure::types::Locator,
        ) {
            self.received_messages
                .push((destination_locator, to_bytes(message).unwrap()));
        }
    }

    impl<'a> TransportRead<'a> for Mailbox {
        fn read(
            &'a mut self,
        ) -> Option<(
            rtps_pim::structure::types::Locator,
            dds_transport::RtpsMessage<'a>,
        )> {
            if self.read < self.received_messages.len() {
                let (locator, message_bytes) = &self.received_messages[self.read];
                self.read += 1;
                Some((
                    locator.clone(),
                    from_bytes(message_bytes.as_slice()).unwrap(),
                ))
            } else {
                None
            }
        }
    }

    fn mock_locator(i: u8) -> Locator {
        Locator {
            kind: LOCATOR_KIND_UDPv4,
            port: 42,
            address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, i],
        }
    }

    #[test]
    fn test_spdp_send_receive() {
        let domain_id = 8;
        let guid_prefix = GuidPrefix([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let metatraffic_unicast_locator_list = vec![mock_locator(0)];
        let metatraffic_multicast_locator_list = vec![mock_locator(1)];
        let default_unicast_locator_list = vec![mock_locator(2), mock_locator(3), mock_locator(4)];

        // ////////// Create 2 participants

        let participant1 = DomainParticipantImpl::new(
            guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            metatraffic_unicast_locator_list.clone(),
            metatraffic_multicast_locator_list.clone(),
            default_unicast_locator_list.clone(),
            vec![],
        );
        participant1.enable().unwrap();
        participant1.create_builtins().unwrap();

        let participant2 = DomainParticipantImpl::new(
            GUIDPREFIX_UNKNOWN,
            0 as i32,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        );
        participant2.enable().unwrap();
        participant2.create_builtins().unwrap();

        // ////////// Send and receive SPDP data
        {
            let mut spdp_mailbox = Mailbox::new();

            participant1.announce_participant().unwrap();
            participant1.send_built_in_data(&mut spdp_mailbox);
            participant2.receive_built_in_data(&mut spdp_mailbox);
        }

        // ////////// Participant 2 receives discovered participant data
        let spdp_discovered_participant_data_sample = {
            let participant2_proxy = DomainParticipant::new(participant2.downgrade());

            let subscriber = Subscriber::new(
                participant2
                    .get_builtin_subscriber()
                    .as_ref()
                    .unwrap()
                    .downgrade(),
            );

            let participant_topic: Topic<SpdpDiscoveredParticipantData> = participant2_proxy
                .lookup_topicdescription(DCPS_PARTICIPANT)
                .unwrap();
            let participant2_builtin_participant_data_reader =
                subscriber.lookup_datareader(&participant_topic).unwrap();

            &participant2_builtin_participant_data_reader
                .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
                .unwrap()[0]
        };

        // ////////// Check that the received data is correct
        {
            assert_eq!(
                BuiltInTopicKey {
                    value: Guid::new(guid_prefix, ENTITYID_PARTICIPANT).into()
                },
                spdp_discovered_participant_data_sample
                    .data
                    .as_ref()
                    .unwrap()
                    .dds_participant_data
                    .key,
            );

            assert_eq!(
                domain_id,
                spdp_discovered_participant_data_sample
                    .data
                    .as_ref()
                    .unwrap()
                    .participant_proxy
                    .domain_id
            );

            assert_eq!(
                guid_prefix,
                spdp_discovered_participant_data_sample
                    .data
                    .as_ref()
                    .unwrap()
                    .participant_proxy
                    .guid_prefix
            );

            assert_eq!(
                metatraffic_unicast_locator_list,
                spdp_discovered_participant_data_sample
                    .data
                    .as_ref()
                    .unwrap()
                    .participant_proxy
                    .metatraffic_unicast_locator_list
            );

            assert_eq!(
                metatraffic_multicast_locator_list,
                spdp_discovered_participant_data_sample
                    .data
                    .as_ref()
                    .unwrap()
                    .participant_proxy
                    .metatraffic_multicast_locator_list
            );

            assert_eq!(
                default_unicast_locator_list,
                spdp_discovered_participant_data_sample
                    .data
                    .as_ref()
                    .unwrap()
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
        fn deserialize(buf: &mut &'de [u8]) -> crate::return_type::DdsResult<Self> {
            Ok(UserData(buf[0]))
        }
    }

    impl DdsSerialize for UserData {
        fn serialize<W: std::io::Write, E: crate::dds_type::Endianness>(
            &self,
            mut writer: W,
        ) -> crate::return_type::DdsResult<()> {
            writer
                .write(&[self.0])
                .map(|_| ())
                .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
        }
    }

    #[test]
    fn test_sedp_send_receive() {
        // ////////// Create 2 participants
        let participant1 = DomainParticipantImpl::new(
            GUIDPREFIX_UNKNOWN,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![mock_locator(0)],
            vec![mock_locator(1)],
            vec![mock_locator(2)],
            vec![],
        );
        participant1.enable().unwrap();
        let participant1_proxy = DomainParticipant::new(participant1.downgrade());
        participant1.create_builtins().unwrap();

        let participant2 = DomainParticipantImpl::new(
            GUIDPREFIX_UNKNOWN,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![mock_locator(3)],
            vec![mock_locator(4)],
            vec![mock_locator(5)],
            vec![],
        );
        participant2.enable().unwrap();
        let participant2_proxy = DomainParticipant::new(participant2.downgrade());
        participant2.create_builtins().unwrap();

        // Match SEDP endpoints
        {
            let mut spdp_mailbox1 = Mailbox::new();
            let mut spdp_mailbox2 = Mailbox::new();

            participant1.announce_participant().unwrap();
            participant2.announce_participant().unwrap();

            participant1.send_built_in_data(&mut spdp_mailbox2);
            participant2.send_built_in_data(&mut spdp_mailbox1);

            participant1.receive_built_in_data(&mut spdp_mailbox1);
            participant2.receive_built_in_data(&mut spdp_mailbox2);

            participant1.discover_matched_participants().unwrap();
            participant2.discover_matched_participants().unwrap();
        }

        // ////////// Create user endpoints
        let user_publisher = participant1_proxy.create_publisher(None, None, 0).unwrap();
        let user_subscriber = participant1_proxy.create_subscriber(None, None, 0).unwrap();

        let user_topic = participant1_proxy
            .create_topic::<UserData>("UserTopic", None, None, 0)
            .unwrap();
        let _user_writer = user_publisher
            .create_datawriter(&user_topic, None, None, 0)
            .unwrap();
        let _user_reader = user_subscriber
            .create_datareader(&user_topic, None, None, 0)
            .unwrap();

        // ////////// Send and receive SEDP data
        {
            let mut sedp_mailbox1 = Mailbox::new();
            let mut sedp_mailbox2 = Mailbox::new();

            participant1.send_built_in_data(&mut sedp_mailbox2);
            participant2.send_built_in_data(&mut sedp_mailbox1);

            participant1.receive_built_in_data(&mut sedp_mailbox1);
            participant2.receive_built_in_data(&mut sedp_mailbox2);
        }

        // ////////// Check that the received data corresponds to the sent data

        let sedp_topic_publication: Topic<DiscoveredWriterData> = participant2_proxy
            .lookup_topicdescription(DCPS_PUBLICATION)
            .unwrap();
        let sedp_topic_subscription: Topic<DiscoveredReaderData> = participant2_proxy
            .lookup_topicdescription(DCPS_SUBSCRIPTION)
            .unwrap();
        let sedp_topic_topic: Topic<DiscoveredTopicData> = participant2_proxy
            .lookup_topicdescription(DCPS_TOPIC)
            .unwrap();

        let participant2_subscriber = Subscriber::new(
            participant2
                .get_builtin_subscriber()
                .as_ref()
                .unwrap()
                .downgrade(),
        );

        let _participant2_publication_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_publication)
            .unwrap();
        let _participant2_subscription_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_subscription)
            .unwrap();
        let participant2_topic_datareader = participant2_subscriber
            .lookup_datareader(&sedp_topic_topic)
            .unwrap();

        let discovered_topic_data_sample = &participant2_topic_datareader
            .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap()[0];
        assert_eq!(
            UserData::type_name(),
            discovered_topic_data_sample
                .data
                .as_ref()
                .unwrap()
                .topic_builtin_topic_data
                .type_name,
        );
        assert_eq!(
            user_topic.get_name().unwrap(),
            discovered_topic_data_sample
                .data
                .as_ref()
                .unwrap()
                .topic_builtin_topic_data
                .name,
        );
    }

    mock! {
        #[derive(Clone)]
        ReaderListener {}

        impl DataReaderListener for ReaderListener {
            type Foo = UserData;
            fn on_subscription_matched(&mut self, the_reader: &DataReader<UserData>, status: SubscriptionMatchedStatus);
            fn on_data_available(&mut self, the_reader: &DataReader<UserData>);
        }
    }

    mock! {
        #[derive(Clone)]
        WriterListener {}

        impl DataWriterListener for WriterListener {
            type Foo = UserData;

            fn on_publication_matched(
                &mut self,
                the_writer: &DataWriterProxy<UserData>,
                status: PublicationMatchedStatus,
            );

            fn on_liveliness_lost(
                &mut self,
                _the_writer: &DataWriterProxy<UserData>,
                _status: crate::dcps_psm::LivelinessLostStatus,
            );

            fn on_offered_deadline_missed(
                &mut self,
                _the_writer: &DataWriterProxy<UserData>,
                _status: crate::dcps_psm::OfferedDeadlineMissedStatus,
            );

            fn on_offered_incompatible_qos(
                &mut self,
                _the_writer: &DataWriterProxy<UserData>,
                _status: crate::dcps_psm::OfferedIncompatibleQosStatus,
            );
        }
    }

    #[test]
    fn test_reader_writer_matching_listener() {
        // ////////// Create 2 participants
        let participant1 = DomainParticipantImpl::new(
            GUIDPREFIX_UNKNOWN,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![mock_locator(0)],
            vec![mock_locator(1)],
            vec![mock_locator(2)],
            vec![],
        );
        participant1.enable().unwrap();
        let participant1_proxy = DomainParticipant::new(participant1.downgrade());
        participant1.create_builtins().unwrap();

        let participant2 = DomainParticipantImpl::new(
            GUIDPREFIX_UNKNOWN,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![mock_locator(3)],
            vec![mock_locator(4)],
            vec![mock_locator(5)],
            vec![],
        );
        participant2.enable().unwrap();
        let participant2_proxy = DomainParticipant::new(participant2.downgrade());
        participant2.create_builtins().unwrap();

        // ////////// Match SEDP endpoints
        {
            let mut spdp_mailbox1 = Mailbox::new();
            let mut spdp_mailbox2 = Mailbox::new();

            participant1.announce_participant().unwrap();
            participant2.announce_participant().unwrap();

            participant1.send_built_in_data(&mut spdp_mailbox2);
            participant2.send_built_in_data(&mut spdp_mailbox1);

            participant1.receive_built_in_data(&mut spdp_mailbox1);
            participant2.receive_built_in_data(&mut spdp_mailbox2);

            participant1.discover_matched_participants().unwrap();
            participant2.discover_matched_participants().unwrap();
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
            let mut sedp_mailbox1 = Mailbox::new();
            let mut sedp_mailbox2 = Mailbox::new();

            participant1.send_built_in_data(&mut sedp_mailbox2);
            participant2.send_built_in_data(&mut sedp_mailbox1);

            participant1.receive_built_in_data(&mut sedp_mailbox1);
            participant2.receive_built_in_data(&mut sedp_mailbox2);
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

            participant1.discover_matched_readers().unwrap();

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

            participant2.discover_matched_writers().unwrap();

            user_reader
                .set_listener(Some(Box::new(MockReaderListener::new())), 0)
                .unwrap();
        }
    }

    #[test]
    fn test_reader_available_data_listener() {
        // ////////// Create 2 participants
        let participant1 = DomainParticipantImpl::new(
            GUIDPREFIX_UNKNOWN,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![mock_locator(0)],
            vec![mock_locator(1)],
            vec![mock_locator(2)],
            vec![],
        );
        participant1.enable().unwrap();
        let participant1_proxy = DomainParticipant::new(participant1.downgrade());
        participant1.create_builtins().unwrap();

        let participant2 = DomainParticipantImpl::new(
            GUIDPREFIX_UNKNOWN,
            0,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![mock_locator(3)],
            vec![mock_locator(4)],
            vec![mock_locator(5)],
            vec![],
        );
        participant2.enable().unwrap();
        let participant2_proxy = DomainParticipant::new(participant2.downgrade());
        participant2.create_builtins().unwrap();

        // ////////// Match SEDP endpoints
        {
            let mut spdp_mailbox1 = Mailbox::new();
            let mut spdp_mailbox2 = Mailbox::new();

            participant1.announce_participant().unwrap();
            participant2.announce_participant().unwrap();

            participant1.send_built_in_data(&mut spdp_mailbox2);
            participant2.send_built_in_data(&mut spdp_mailbox1);

            participant1.receive_built_in_data(&mut spdp_mailbox1);
            participant2.receive_built_in_data(&mut spdp_mailbox2);

            participant1.discover_matched_participants().unwrap();
            participant2.discover_matched_participants().unwrap();
        }

        // ////////// Create user endpoints
        let user_publisher = participant1_proxy.create_publisher(None, None, 0).unwrap();
        let user_subscriber = participant2_proxy.create_subscriber(None, None, 0).unwrap();

        let user_topic = participant1_proxy
            .create_topic::<UserData>("UserTopic", None, None, 0)
            .unwrap();
        let user_writer = user_publisher
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
            let mut sedp_mailbox1 = Mailbox::new();
            let mut sedp_mailbox2 = Mailbox::new();

            participant1.send_built_in_data(&mut sedp_mailbox2);
            participant2.send_built_in_data(&mut sedp_mailbox1);

            participant1.receive_built_in_data(&mut sedp_mailbox1);
            participant2.receive_built_in_data(&mut sedp_mailbox2);

            // ////////// Process SEDP data
            participant1.discover_matched_readers().unwrap();

            // We expect the subscription matched listener to be called when matching
            let mut reader_listener = Box::new(MockReaderListener::new());
            reader_listener
                .expect_on_subscription_matched()
                .return_const(());
            user_reader.set_listener(Some(reader_listener), 0).unwrap();

            participant2.discover_matched_writers().unwrap();

            // No more listener should be called for now
            user_reader
                .set_listener(Some(Box::new(MockReaderListener::new())), 0)
                .unwrap();
        }

        // ////////// Write user data
        user_writer.write(&UserData(8), None).unwrap();

        // ////////// Send user data
        {
            let mut user_mailbox = Mailbox::new();

            participant1.send_user_defined_data(&mut user_mailbox);

            // On receive the available data listener should be called
            let mut reader_listener = Box::new(MockReaderListener::new());
            reader_listener
                .expect_on_data_available()
                .once()
                .return_const(());
            user_reader.set_listener(Some(reader_listener), 0).unwrap();

            participant2.receive_user_defined_data(&mut user_mailbox);

            // From now on no listener should be called anymore
            user_reader
                .set_listener(Some(Box::new(MockReaderListener::new())), 0)
                .unwrap();
        }
    }

    fn reset_singleton() {
        THE_PARTICIPANT_FACTORY
            .participant_list
            .lock()
            .unwrap()
            .clear();
    }

    #[test]
    fn test_delete_participant() {
        reset_singleton();

        let participant_factory = DomainParticipantFactory::get_instance();

        let participant1 = participant_factory
            .create_participant(1, None, None, 0)
            .unwrap();

        let participant2 = participant_factory
            .create_participant(2, None, None, 0)
            .unwrap();

        participant_factory
            .delete_participant(&participant1)
            .unwrap();

        assert_eq!(
            1,
            THE_PARTICIPANT_FACTORY
                .participant_list
                .lock()
                .unwrap()
                .len()
        );

        assert!(participant1.enable().is_err());
        assert!(participant2.enable().is_ok());
    }
}
