use super::domain_participant::DomainParticipant;
use crate::{
    configuration::DustDdsConfiguration,
    dcps::domain_participant_factory_actor::DdsTransportParticipantFactory,
    dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::{
        domain::DomainId,
        error::DdsResult,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::DdsRuntime,
};
use tracing::warn;

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// [`DomainParticipantFactory`] itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory<R: DdsRuntime> {
    participant_factory_async: &'static DomainParticipantFactoryAsync<R>,
}

impl<R: DdsRuntime> DomainParticipantFactory<R> {
    /// This operation creates a new [`DomainParticipant`] object. The [`DomainParticipant`] signifies that the calling application intends
    /// to join the Domain identified by the `domain_id` argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no [`DomainParticipant`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`DomainParticipant`] should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation [`DomainParticipantFactory::get_default_participant_qos`] and using the resulting
    /// QoS to create the [`DomainParticipant`].
    #[tracing::instrument(skip(self, a_listener))]
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<impl DomainParticipantListener<R> + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant<R>> {
        R::block_on(
            self.participant_factory_async
                .create_participant(domain_id, qos, a_listener, mask),
        )
        .map(DomainParticipant::new)
    }

    /// This operation deletes an existing [`DomainParticipant`]. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted otherwise the error [`DdsError::PreconditionNotMet`](crate::infrastructure::error::DdsError::PreconditionNotMet) is returned. If the
    /// participant has been previously deleted this operation returns the error [`DdsError::AlreadyDeleted`](crate::infrastructure::error::DdsError::AlreadyDeleted).
    #[tracing::instrument(skip(self, participant))]
    pub fn delete_participant(&self, participant: &DomainParticipant<R>) -> DdsResult<()> {
        R::block_on(
            self.participant_factory_async
                .delete_participant(participant.participant_async()),
        )
    }

    /// This operation retrieves a previously created [`DomainParticipant`] belonging to the specified domain_id. If no such
    /// [`DomainParticipant`] exists, the operation will return a [`None`] value.
    /// If multiple [`DomainParticipant`] entities belonging to that domain_id exist, then the operation will return one of them. It is not
    /// specified which one.
    #[tracing::instrument(skip(self))]
    pub fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> DdsResult<Option<DomainParticipant<R>>> {
        Ok(
            R::block_on(self.participant_factory_async.lookup_participant(domain_id))?
                .map(DomainParticipant::new),
        )
    }

    /// This operation sets a default value of the [`DomainParticipantQos`] policies which will be used for newly created
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`](crate::infrastructure::error::DdsError::InconsistentPolicy).
    #[tracing::instrument(skip(self))]
    pub fn set_default_participant_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        R::block_on(
            self.participant_factory_async
                .set_default_participant_qos(qos),
        )
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`]
    /// operation.
    /// The values retrieved by [`DomainParticipantFactory::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`DomainParticipantFactory::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    #[tracing::instrument(skip(self))]
    pub fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        R::block_on(self.participant_factory_async.get_default_participant_qos())
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`](crate::infrastructure::error::DdsError::InconsistentPolicy).
    #[tracing::instrument(skip(self))]
    pub fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        R::block_on(self.participant_factory_async.set_qos(qos))
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    #[tracing::instrument(skip(self))]
    pub fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        R::block_on(self.participant_factory_async.get_qos())
    }
}

impl<R: DdsRuntime> DomainParticipantFactory<R> {
    /// Set the configuration of the [`DomainParticipantFactory`] singleton
    pub fn set_configuration(&self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        R::block_on(
            self.participant_factory_async
                .set_configuration(configuration),
        )
    }

    /// Get the current configuration of the [`DomainParticipantFactory`] singleton
    pub fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        R::block_on(self.participant_factory_async.get_configuration())
    }

    /// Set the transport to be used by the [`DomainParticipants`] entities
    /// created by the [`DomainParticipantFactory`] singleton
    pub fn set_transport(&self, transport: DdsTransportParticipantFactory) -> DdsResult<()> {
        R::block_on(self.participant_factory_async.set_transport(transport))
    }
}

#[cfg(feature = "std")]
impl DomainParticipantFactory<crate::std_runtime::StdRuntime> {
    /// This operation returns the [`DomainParticipantFactory`] singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same [`DomainParticipantFactory`] instance.
    #[tracing::instrument]
    pub fn get_instance() -> &'static Self {
        static PARTICIPANT_FACTORY: std::sync::OnceLock<
            DomainParticipantFactory<crate::std_runtime::StdRuntime>,
        > = std::sync::OnceLock::new();
        PARTICIPANT_FACTORY.get_or_init(|| DomainParticipantFactory {
            participant_factory_async: DomainParticipantFactoryAsync::<
                crate::std_runtime::StdRuntime,
            >::get_instance(),
        })
    }
}
