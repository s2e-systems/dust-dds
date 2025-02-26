use std::sync::OnceLock;

use super::{
    domain_participant::DomainParticipantAsync,
    domain_participant_listener::DomainParticipantListenerAsync,
};
use crate::{
    configuration::DustDdsConfiguration,
    domain::domain_participant_factory::DomainId,
    implementation::{
        domain_participant_backend::services::{discovery_service, domain_participant_service},
        domain_participant_factory::domain_participant_factory_actor::{
            self, DomainParticipantFactoryActor,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::{actor::Actor, executor::Executor, timer::TimerDriver},
    transport::factory::TransportParticipantFactory,
};

/// Async version of [`DomainParticipantFactory`](crate::domain::domain_participant_factory::DomainParticipantFactory).
/// Unlike the sync version, the [`DomainParticipantFactoryAsync`] is not a singleton and can be created by means of
/// a constructor by passing a handle to a [`Tokio`](https://crates.io/crates/tokio) runtime. This allows the factory
/// to spin tasks on an existing runtime which can be shared with other things outside Dust DDS.
pub struct DomainParticipantFactoryAsync {
    _executor: Executor,
    timer_driver: TimerDriver,
    domain_participant_factory_actor: Actor<DomainParticipantFactoryActor>,
}

impl DomainParticipantFactoryAsync {
    /// Async version of [`create_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::create_participant).
    pub async fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListenerAsync + Send + 'static>>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipantAsync> {
        let status_kind = mask.to_vec();
        let (
            participant_address,
            participant_handle,
            participant_status_condition_address,
            builtin_subscriber_status_condition_address,
        ) = self
            .domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::CreateParticipant {
                domain_id,
                qos,
                listener: a_listener,
                status_kind,
            })
            .receive_reply()
            .await?;

        let domain_participant = DomainParticipantAsync::new(
            participant_address.clone(),
            participant_status_condition_address,
            builtin_subscriber_status_condition_address,
            domain_id,
            participant_handle,
            self.timer_driver.handle(),
        );

        Ok(domain_participant)
    }

    /// Async version of [`delete_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::delete_participant).
    pub async fn delete_participant(&self, participant: &DomainParticipantAsync) -> DdsResult<()> {
        let is_participant_empty = participant
            .participant_address()
            .send_actor_mail(domain_participant_service::IsEmpty)?
            .receive_reply()
            .await;
        if is_participant_empty {
            let handle = participant.get_instance_handle().await;

            let deleted_participant = self
                .domain_participant_factory_actor
                .send_actor_mail(domain_participant_factory_actor::DeleteParticipant { handle })
                .receive_reply()
                .await?;
            deleted_participant
                .send_actor_mail(discovery_service::AnnounceDeletedParticipant)
                .receive_reply()
                .await?;
            deleted_participant.stop().await;
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    /// This operation returns the [`DomainParticipantFactoryAsync`] singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same [`DomainParticipantFactoryAsync`] instance.
    #[tracing::instrument]
    pub fn get_instance() -> &'static Self {
        static PARTICIPANT_FACTORY_ASYNC: OnceLock<DomainParticipantFactoryAsync> = OnceLock::new();
        PARTICIPANT_FACTORY_ASYNC.get_or_init(|| {
            let executor = Executor::new();
            let timer_driver = TimerDriver::new();
            let domain_participant_factory_actor =
                Actor::spawn(DomainParticipantFactoryActor::new(), &executor.handle());
            Self {
                _executor: executor,
                domain_participant_factory_actor,
                timer_driver,
            }
        })
    }

    /// Async version of [`lookup_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::lookup_participant).
    pub async fn lookup_participant(
        &self,
        _domain_id: DomainId,
    ) -> DdsResult<Option<DomainParticipantAsync>> {
        todo!()
    }

    /// Async version of [`set_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_default_participant_qos).
    pub async fn set_default_participant_qos(
        &self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::SetDefaultParticipantQos { qos })
            .receive_reply()
            .await
    }

    /// Async version of [`get_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_default_participant_qos).
    pub async fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        Ok(self
            .domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::GetDefaultParticipantQos)
            .receive_reply()
            .await)
    }

    /// Async version of [`set_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_qos).
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::SetQos { qos })
            .receive_reply()
            .await
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_qos).
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        Ok(self
            .domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::GetQos)
            .receive_reply()
            .await)
    }

    /// Async version of [`set_configuration`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_configuration).
    pub async fn set_configuration(&self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::SetConfiguration { configuration })
            .receive_reply()
            .await;
        Ok(())
    }

    /// Async version of [`get_configuration`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_configuration).
    pub async fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        Ok(self
            .domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::GetConfiguration)
            .receive_reply()
            .await)
    }

    /// Async version of [`set_transport`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_transport).
    pub async fn set_transport(
        &self,
        transport: Box<dyn TransportParticipantFactory>,
    ) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::SetTransport { transport })
            .receive_reply()
            .await;
        Ok(())
    }
}
