use std::sync::Mutex;

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
    infrastructure::{
        error::{DdsError, DdsResult},
        status::StatusKind,
    },
};
use crate::{
    implementation::{
        rtps::{
            participant::RtpsParticipant,
            types::{GuidPrefix, PROTOCOLVERSION, VENDOR_ID_S2E},
        },
        rtps_udp_psm::udp_transport::RtpsUdpPsm,
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos},
    },
};

use lazy_static::lazy_static;

use crate::domain::domain_participant_listener::DomainParticipantListener;

use super::{configuration::DustDdsConfiguration, domain_participant::DomainParticipant};

type DomainIdTypeNative = i32;
pub type DomainId = DomainIdTypeNative;

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
    /// This value can be used as an alias for the singleton factory returned by the operation
    /// [`DomainParticipantFactory::get_instance()`].
    pub static ref THE_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory {
        participant_list: Mutex::new(vec![])
    };
}

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// DomainParticipantFactory itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory {
    participant_list: Mutex<Vec<ParticipantManager>>,
}

impl DomainParticipantFactory {
    /// This operation creates a new [`DomainParticipant`] object. The [`DomainParticipant`] signifies that the calling application intends
    /// to join the Domain identified by the domain_id argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no [`DomainParticipant`] will be created.
    /// The special value PARTICIPANT_QOS_DEFAULT can be used to indicate that the DomainParticipant should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation [`Self::get_default_participant_qos`] and using the resulting
    /// QoS to create the [`DomainParticipant`].
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant> {
        let configuration = DustDdsConfiguration::try_from_environment_variable()?;

        let qos = qos.unwrap_or_default();

        let rtps_udp_psm = RtpsUdpPsm::new(domain_id).map_err(DdsError::PreconditionNotMet)?;
        let rtps_participant = RtpsParticipant::new(
            GuidPrefix(rtps_udp_psm.guid_prefix()),
            rtps_udp_psm.default_unicast_locator_list().as_ref(),
            rtps_udp_psm.default_multicast_locator_list(),
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
        );
        let domain_participant = DomainParticipantImpl::new(
            rtps_participant,
            domain_id,
            configuration.domain_tag,
            qos.clone(),
            rtps_udp_psm.metatraffic_unicast_locator_list(),
            rtps_udp_psm.metatraffic_multicast_locator_list(),
        );

        if qos.entity_factory.autoenable_created_entities {
            domain_participant.enable()?;
        }

        domain_participant.create_builtins()?;

        let mut participant_manager = ParticipantManager {
            participant: domain_participant,
            task_manager: TaskManager::new(),
        };
        participant_manager.start(rtps_udp_psm);

        let participant = DomainParticipant::new(participant_manager.participant.downgrade());

        THE_PARTICIPANT_FACTORY
            .participant_list
            .lock()
            .unwrap()
            .push(participant_manager);

        Ok(participant)
    }

    /// This operation deletes an existing [`DomainParticipant`]. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted otherwise the error [`DdsError::PreconditionNotMet`] is returned. If the
    /// participant has been previously deleted this operation returns the error [`DdsError::AlreadyDeleted`].
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

    /// This operation returns the [`DomainParticipantFactory`] singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same [`DomainParticipantFactory`] instance.
    /// The pre-defined value [`struct@THE_PARTICIPANT_FACTORY`] can also be used as an alias for the singleton factory returned by this operation.
    pub fn get_instance() -> &'static Self {
        &THE_PARTICIPANT_FACTORY
    }

    /// This operation retrieves a previously created [`DomainParticipant`] belonging to the specified domain_id. If no such
    /// [`DomainParticipant`] exists, the operation will return a [`None`] value.
    /// If multiple [`DomainParticipant`] entities belonging to that domain_id exist, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_participant(&self, _domain_id: DomainId) -> Option<DomainParticipant> {
        todo!()
    }

    /// This operation sets a default value of the [`DomainParticipantQos`] policies which will be used for newly created
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`Self::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_default_participant_qos(&self, _qos: Option<DomainParticipantQos>) -> DdsResult<()> {
        todo!()
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`Self::create_participant`]
    /// operation.
    /// The values retrieved by [`Self::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`Self::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    pub fn get_default_participant_qos(&self) -> DomainParticipantQos {
        todo!()
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_qos(&self, _qos: Option<DomainParticipantFactoryQos>) -> DdsResult<()> {
        todo!()
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    pub fn get_qos(&self) -> DomainParticipantFactoryQos {
        todo!()
    }
}

impl DomainParticipantFactory {
    /// This functions returns the participant which is the parent
    /// of the object passed as instance handle.
    pub(crate) fn lookup_participant_by_entity_handle(
        &self,
        instance_handle: InstanceHandle,
    ) -> DdsShared<DomainParticipantImpl> {
        let participant_list_lock = self.participant_list.lock().unwrap();
        participant_list_lock
            .iter()
            .find(|x| {
                if let Ok(handle) = x.participant.get_instance_handle() {
                    <[u8; 16]>::from(handle)[0..12] == <[u8; 16]>::from(instance_handle)[0..12]
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
