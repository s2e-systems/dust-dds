use crate::{
    configuration::DustDdsConfiguration,
    domain::domain_participant_factory::DomainId,
    implementation::{
        actors::{
            domain_participant_actor,
            domain_participant_factory_actor::{self, DomainParticipantFactoryActor},
            subscriber_actor,
        },
        utils::actor::Actor,
    },
    infrastructure::{
        error::DdsResult,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};

use super::{
    domain_participant::DomainParticipantAsync,
    domain_participant_listener::DomainParticipantListenerAsync,
};

/// Async version of [`DomainParticipantFactory`](crate::domain::domain_participant_factory::DomainParticipantFactory).
/// Unlike the sync version, the [`DomainParticipantFactoryAsync`] is not a singleton and can be created by means of
/// a constructor by passing a handle to a [`Tokio`](https://crates.io/crates/tokio) runtime. This allows the factory
/// to spin tasks on an existing runtime which can be shared with other things outside Dust DDS.
pub struct DomainParticipantFactoryAsync {
    domain_participant_factory_actor: Actor<DomainParticipantFactoryActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl DomainParticipantFactoryAsync {
    /// Create a new [`DomainParticipantFactoryAsync`].
    /// All the tasks of Dust DDS will be spawned on the runtime which is given as an argument.
    pub fn new(runtime_handle: tokio::runtime::Handle) -> Self {
        let domain_participant_factory_actor =
            Actor::spawn(DomainParticipantFactoryActor::new(), &runtime_handle);

        Self {
            domain_participant_factory_actor,
            runtime_handle,
        }
    }

    /// Async version of [`create_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::create_participant).
    pub async fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: impl DomainParticipantListenerAsync + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipantAsync> {
        let listener = Box::new(a_listener);
        let status_kind = mask.to_vec();
        let runtime_handle = self.runtime_handle.clone();
        let participant_address = self
            .domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::create_participant::new(
                domain_id,
                qos,
                listener,
                status_kind,
                runtime_handle,
            ))
            .await?;
        let status_condition = participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_statuscondition::new())
            .await?;
        let builtin_subscriber = participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_built_in_subscriber::new())
            .await?;
        let builtin_subscriber_status_condition_address = builtin_subscriber
            .send_mail_and_await_reply(subscriber_actor::get_statuscondition::new())
            .await?;
        let domain_participant = DomainParticipantAsync::new(
            participant_address.clone(),
            status_condition,
            builtin_subscriber,
            builtin_subscriber_status_condition_address,
            domain_id,
            self.runtime_handle.clone(),
        );

        if self
            .get_qos()
            .await?
            .entity_factory
            .autoenable_created_entities
        {
            domain_participant.enable().await?;
        }

        Ok(domain_participant)
    }

    /// Async version of [`delete_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::delete_participant).
    pub async fn delete_participant(&self, participant: &DomainParticipantAsync) -> DdsResult<()> {
        let handle = participant.get_instance_handle().await?;
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::delete_participant::new(
                handle,
            ))
            .await
    }

    /// Async version of [`lookup_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::lookup_participant).
    pub async fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> DdsResult<Option<DomainParticipantAsync>> {
        if let Some(dp) = self
            .domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::lookup_participant::new(
                domain_id,
            ))
            .await?
        {
            let status_condition = dp
                .send_mail_and_await_reply(domain_participant_actor::get_statuscondition::new())
                .await?;
            let builtin_subscriber = dp
                .send_mail_and_await_reply(domain_participant_actor::get_built_in_subscriber::new())
                .await?;
            let builtin_subscriber_status_condition_address = builtin_subscriber
                .send_mail_and_await_reply(subscriber_actor::get_statuscondition::new())
                .await?;
            Ok(Some(DomainParticipantAsync::new(
                dp,
                status_condition,
                builtin_subscriber,
                builtin_subscriber_status_condition_address,
                domain_id,
                self.runtime_handle.clone(),
            )))
        } else {
            Ok(None)
        }
    }

    /// Async version of [`set_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_default_participant_qos).
    pub async fn set_default_participant_qos(
        &self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(
                domain_participant_factory_actor::set_default_participant_qos::new(qos),
            )
            .await
    }

    /// Async version of [`get_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_default_participant_qos).
    pub async fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(
                domain_participant_factory_actor::get_default_participant_qos::new(),
            )
            .await
    }

    /// Async version of [`set_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_qos).
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::set_qos::new(qos))
            .await
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_qos).
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::get_qos::new())
            .await
    }

    /// Async version of [`set_configuration`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_configuration).
    pub async fn set_configuration(&self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::set_configuration::new(
                configuration,
            ))
            .await
    }

    /// Async version of [`get_configuration`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_configuration).
    pub async fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::get_configuration::new())
            .await
    }
}
