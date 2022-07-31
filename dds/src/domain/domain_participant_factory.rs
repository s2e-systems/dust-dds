use std::sync::Mutex;

use crate::{
    dcps_psm::{DomainId, InstanceHandle, StatusMask},
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
    pub static ref THE_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory {
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

impl DomainParticipantFactory {
    /// This functions returns the participant which is the parent
    /// of the object passed as instance handle.
    pub(crate) fn lookup_participant_by_entity_handle(
        &self,
        instance_handle: &InstanceHandle,
    ) -> DdsShared<DomainParticipantImpl> {
        let participant_list_lock = self.participant_list.lock().unwrap();
        participant_list_lock
            .iter()
            .find(|x| {
                if let Ok(handle) = x.participant.get_instance_handle() {
                    handle[0..12] == instance_handle[0..12]
                } else {
                    false
                }
            })
            .map(|x| x.participant.clone())
            // This function is only used internally and only for valid handles
            // so it should never fail to find an existing participant
            .expect("Failed to find participant in factory")
    }
}
