use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::dds::nodes::DomainParticipantNode,
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};

use lazy_static::lazy_static;

use super::domain_participant::DomainParticipant;

pub type DomainId = i32;

lazy_static! {}

/// This value can be used as an alias for the singleton factory returned by the operation
/// [`DomainParticipantFactory::get_instance()`].
pub static THE_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory;

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// [`DomainParticipantFactory`] itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory;

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
        todo!()
        // let participant = THE_TASK_RUNTIME.block_on(
        //     THE_DDS_DOMAIN_PARTICIPANT_FACTORY.create_participant(domain_id, qos, a_listener, mask),
        // )?;

        // if THE_DDS_DOMAIN_PARTICIPANT_FACTORY
        //     .qos()
        //     .entity_factory
        //     .autoenable_created_entities
        // {
        //     participant.enable()?;
        // }

        // Ok(participant)
    }

    /// This operation deletes an existing [`DomainParticipant`]. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted otherwise the error [`DdsError::PreconditionNotMet`] is returned. If the
    /// participant has been previously deleted this operation returns the error [`DdsError::AlreadyDeleted`].
    pub fn delete_participant(&self, participant: &DomainParticipant) -> DdsResult<()> {
        todo!()
        // let is_participant_empty = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(
        //     &participant.node().guid().prefix(),
        //     |dp| {
        //         let dp = dp.ok_or(DdsError::AlreadyDeleted)?;
        //         Ok(dp.user_defined_publisher_list().iter().count() == 0
        //             && dp.user_defined_subscriber_list().iter().count() == 0
        //             && dp.topic_list().iter().count() == 0)
        //     },
        // )?;

        // if is_participant_empty {
        //     // Explicit external drop to avoid deadlock. Otherwise objects contained in the factory can't access it due to it being blocked
        //     // while locking
        //     let object = THE_DDS_DOMAIN_PARTICIPANT_FACTORY
        //         .domain_participant_list
        //         .blocking_write()
        //         .remove(&participant.node().guid().prefix());
        //     std::mem::drop(object);

        //     Ok(())
        // } else {
        //     Err(DdsError::PreconditionNotMet(
        //         "Domain participant still contains other entities".to_string(),
        //     ))
        // }
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
        todo!()
        // THE_DDS_DOMAIN_PARTICIPANT_FACTORY
        //     .domain_participant_list
        //     .blocking_read()
        //     .iter()
        //     .find_map(|(_, dp)| {
        //         if dp.get_domain_id() == domain_id {
        //             Some(dp.guid())
        //         } else {
        //             None
        //         }
        //     })
        //     .map(|guid| DomainParticipant::new(DomainParticipantNode::new(guid)))
    }

    /// This operation sets a default value of the [`DomainParticipantQos`] policies which will be used for newly created
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_default_participant_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        todo!()
        // let q = match qos {
        //     QosKind::Default => DomainParticipantQos::default(),
        //     QosKind::Specific(q) => q,
        // };

        // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.set_default_participant_qos(q);

        // Ok(())
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`]
    /// operation.
    /// The values retrieved by [`DomainParticipantFactory::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`DomainParticipantFactory::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    pub fn get_default_participant_qos(&self) -> DomainParticipantQos {
        todo!()
        // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.default_participant_qos()
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        todo!()
        // let q = match qos {
        //     QosKind::Default => DomainParticipantFactoryQos::default(),
        //     QosKind::Specific(q) => q,
        // };

        // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.set_qos(q);

        // Ok(())
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    pub fn get_qos(&self) -> DomainParticipantFactoryQos {
        todo!()
        // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.qos()
    }
}
