use std::str::FromStr;

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        dds_impl::{configuration::DustDdsConfiguration, dcps_service::DcpsService},
        rtps::{
            participant::RtpsParticipant,
            types::{GuidPrefix, PROTOCOLVERSION, VENDOR_ID_S2E},
        },
        rtps_udp_psm::udp_transport::RtpsUdpPsm,
        utils::shared_object::DdsRwLock,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};
use lazy_static::lazy_static;
use mac_address::MacAddress;

use super::domain_participant::DomainParticipant;

pub type DomainId = i32;

lazy_static! {
    /// This value can be used as an alias for the singleton factory returned by the operation
    /// [`DomainParticipantFactory::get_instance()`].
    pub static ref THE_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory {
        participant_list: DdsRwLock::new(vec![]),
        qos: DdsRwLock::new(DomainParticipantFactoryQos::default()),
        default_participant_qos: DdsRwLock::new(DomainParticipantQos::default()),
    };
}

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// [`DomainParticipantFactory`] itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory {
    participant_list: DdsRwLock<Vec<DcpsService>>,
    qos: DdsRwLock<DomainParticipantFactoryQos>,
    default_participant_qos: DdsRwLock<DomainParticipantQos>,
}

impl DomainParticipantFactory {
    /// This operation creates a new [`DomainParticipant`] object. The [`DomainParticipant`] signifies that the calling application intends
    /// to join the Domain identified by the `domain_id` argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no [`DomainParticipant`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`DomainParticipant`] should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation [`DomainParticipantFactory::get_default_participant_qos`] and using the resulting
    /// QoS to create the [`DomainParticipant`].
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant> {
        let configuration = if let Ok(configuration_json) = std::env::var("DUST_DDS_CONFIGURATION")
        {
            DustDdsConfiguration::try_from_str(configuration_json.as_str())?
        } else {
            Default::default()
        };

        let qos = match qos {
            QosKind::Default => self.default_participant_qos.read_lock().clone(),
            QosKind::Specific(q) => q,
        };

        let mut participant_list_lock = self.participant_list.write_lock();
        let participant_id = participant_list_lock
            .iter()
            .filter(|p| p.participant().get_domain_id() == domain_id)
            .count();

        let rtps_udp_psm = RtpsUdpPsm::new(
            domain_id,
            participant_id,
            configuration.interface_name.as_ref(),
        )
        .map_err(DdsError::PreconditionNotMet)?;

        let mac_address = ifcfg::IfCfg::get()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter_map(|i| MacAddress::from_str(&i.mac).ok())
            .find(|&mac| mac != MacAddress::new([0, 0, 0, 0, 0, 0]))
            .expect("Could not find any mac address")
            .bytes();

        #[rustfmt::skip]
        let guid_prefix = GuidPrefix::new([
            mac_address[0], mac_address[1], mac_address[2],
            mac_address[3], mac_address[4], mac_address[5],
            domain_id as u8, participant_id as u8, 0, 0, 0, 0
        ]);

        let rtps_participant = RtpsParticipant::new(
            guid_prefix,
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
            a_listener,
            mask,
            rtps_udp_psm,
        )?;

        let participant = DomainParticipant::new(dcps_service.participant().downgrade());

        if self
            .qos
            .read_lock()
            .entity_factory
            .autoenable_created_entities
        {
            participant.enable()?;
        }

        participant_list_lock.push(dcps_service);

        Ok(participant)
    }

    /// This operation deletes an existing [`DomainParticipant`]. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted otherwise the error [`DdsError::PreconditionNotMet`] is returned. If the
    /// participant has been previously deleted this operation returns the error [`DdsError::AlreadyDeleted`].
    pub fn delete_participant(&self, participant: &DomainParticipant) -> DdsResult<()> {
        let mut participant_list = THE_PARTICIPANT_FACTORY.participant_list.write_lock();

        let index = participant_list
            .iter()
            .position(|pm| DomainParticipant::new(pm.participant().downgrade()).eq(participant))
            .ok_or(DdsError::AlreadyDeleted)?;

        if participant_list[index].participant().is_empty() {
            participant_list[index].shutdown_tasks();
            participant_list.remove(index);

            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
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
    pub fn lookup_participant(&self, domain_id: DomainId) -> Option<DomainParticipant> {
        self.participant_list
            .read_lock()
            .iter()
            .find(|dp| dp.participant().get_domain_id() == domain_id)
            .map(|dp| DomainParticipant::new(dp.participant().downgrade()))
    }

    /// This operation sets a default value of the [`DomainParticipantQos`] policies which will be used for newly created
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_default_participant_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => {
                *self.default_participant_qos.write_lock() = DomainParticipantQos::default()
            }
            QosKind::Specific(q) => *self.default_participant_qos.write_lock() = q,
        };

        Ok(())
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`]
    /// operation.
    /// The values retrieved by [`DomainParticipantFactory::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`DomainParticipantFactory::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    pub fn get_default_participant_qos(&self) -> DomainParticipantQos {
        self.default_participant_qos.read_lock().clone()
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => *self.qos.write_lock() = DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => *self.qos.write_lock() = q,
        }

        Ok(())
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    pub fn get_qos(&self) -> DomainParticipantFactoryQos {
        self.qos.read_lock().clone()
    }
}
