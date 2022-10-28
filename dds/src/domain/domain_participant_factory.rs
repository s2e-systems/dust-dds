use std::sync::Mutex;

use crate::{
    implementation::{
        dds_impl::{
            configuration::DustDdsConfiguration, dcps_service::DcpsService,
            domain_participant_impl::DomainParticipantImpl,
        },
        utils::shared_object::DdsShared,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::Qos,
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

use super::domain_participant::DomainParticipant;

type DomainIdTypeNative = i32;
pub type DomainId = DomainIdTypeNative;

lazy_static! {
    /// This value can be used as an alias for the singleton factory returned by the operation
    /// [`DomainParticipantFactory::get_instance()`].
    pub static ref THE_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory {
        participant_list: Mutex::new(vec![]),
        qos: DomainParticipantFactoryQos::default(),
    };
}

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// [`DomainParticipantFactory`] itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory {
    participant_list: Mutex<Vec<DcpsService>>,
    qos: DomainParticipantFactoryQos,
}

impl DomainParticipantFactory {
    /// This operation creates a new [`DomainParticipant`] object. The [`DomainParticipant`] signifies that the calling application intends
    /// to join the Domain identified by the `domain_id` argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no [`DomainParticipant`] will be created.
    /// The special value PARTICIPANT_QOS_DEFAULT can be used to indicate that the [`DomainParticipant`] should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation [`DomainParticipantFactory::get_default_participant_qos`] and using the resulting
    /// QoS to create the [`DomainParticipant`].
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: Qos<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant> {
        let configuration = DustDdsConfiguration::try_from_environment_variable()?;

        let qos = match qos {
            Qos::Default => Default::default(),
            Qos::Specific(q) => q,
        };

        let rtps_udp_psm = RtpsUdpPsm::new(domain_id).map_err(DdsError::PreconditionNotMet)?;
        let rtps_participant = RtpsParticipant::new(
            GuidPrefix(rtps_udp_psm.guid_prefix()),
            rtps_udp_psm.default_unicast_locator_list().as_ref(),
            rtps_udp_psm.default_multicast_locator_list(),
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
        );

        let dcps_service = DcpsService::new(
            rtps_participant,
            domain_id,
            configuration,
            qos,
            rtps_udp_psm,
        )?;

        let participant = DomainParticipant::new(dcps_service.participant().downgrade());

        if self.qos.entity_factory.autoenable_created_entities {
            participant.enable()?;
        }

        participant.create_builtins()?;

        THE_PARTICIPANT_FACTORY
            .participant_list
            .lock()
            .unwrap()
            .push(dcps_service);

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
            .position(|pm| DomainParticipant::new(pm.participant().downgrade()).eq(participant))
            .ok_or(DdsError::AlreadyDeleted)?;

        participant_list[index].shutdown_tasks();
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
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_default_participant_qos(&self, _qos: Qos<DomainParticipantQos>) -> DdsResult<()> {
        todo!()
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`]
    /// operation.
    /// The values retrieved by [`DomainParticipantFactory::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`DomainParticipantFactory::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    pub fn get_default_participant_qos(&self) -> DomainParticipantQos {
        todo!()
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_qos(&self, _qos: Qos<DomainParticipantFactoryQos>) -> DdsResult<()> {
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
                if let Ok(handle) = x.participant().get_instance_handle() {
                    <[u8; 16]>::from(handle)[0..12] == <[u8; 16]>::from(instance_handle)[0..12]
                } else {
                    false
                }
            })
            .map(|x| x.participant().clone())
            // This function is only used internally and only for valid handles
            // so it should never fail to find an existing participant
            .expect("Failed to find participant in factory")
    }
}
