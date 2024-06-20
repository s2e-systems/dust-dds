use crate::{
    configuration::DustDdsConfiguration,
    data_representation_builtin_endpoints::spdp_discovered_participant_data::{
        SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::Actor,
        actors::{
            domain_participant_actor,
            domain_participant_factory_actor::{self, DomainParticipantFactoryActor},
            subscriber_actor,
        },
        runtime::executor::Executor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
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
    _executor: Executor,
    domain_participant_factory_actor: Actor<DomainParticipantFactoryActor>,
}

impl Default for DomainParticipantFactoryAsync {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainParticipantFactoryAsync {
    /// Create a new [`DomainParticipantFactoryAsync`].
    /// All the tasks of Dust DDS will be spawned on the runtime which is given as an argument.
    pub fn new() -> Self {
        let executor = Executor::new();
        let domain_participant_factory_actor =
            Actor::spawn(DomainParticipantFactoryActor::new(), &executor.handle());

        Self {
            _executor: executor,
            domain_participant_factory_actor,
        }
    }

    /// Async version of [`create_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::create_participant).
    pub async fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListenerAsync + Send + 'static>>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipantAsync> {
        let status_kind = mask.to_vec();
        let participant_address = self
            .domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::CreateParticipant {
                domain_id,
                qos,
                listener: a_listener,
                status_kind,
            })
            .receive_reply()
            .await?;
        let status_condition = participant_address
            .send_actor_mail(domain_participant_actor::GetStatuscondition)?
            .receive_reply()
            .await;
        let builtin_subscriber = participant_address
            .send_actor_mail(domain_participant_actor::GetBuiltInSubscriber)?
            .receive_reply()
            .await;
        let builtin_subscriber_status_condition_address = builtin_subscriber
            .send_actor_mail(subscriber_actor::GetStatuscondition)?
            .receive_reply()
            .await;
        let timer_handle = participant_address
            .send_actor_mail(domain_participant_actor::GetTimerHandle)?
            .receive_reply()
            .await;
        let executor_handle = participant_address
            .send_actor_mail(domain_participant_actor::GetExecutorHandle)?
            .receive_reply()
            .await;
        let domain_participant = DomainParticipantAsync::new(
            participant_address.clone(),
            status_condition,
            builtin_subscriber,
            builtin_subscriber_status_condition_address,
            domain_id,
            executor_handle,
            timer_handle,
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
        let is_participant_empty = participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::IsEmpty)?
            .receive_reply()
            .await;
        if is_participant_empty {
            let handle = participant.get_instance_handle().await?;

            let deleted_participant = self
                .domain_participant_factory_actor
                .send_actor_mail(domain_participant_factory_actor::DeleteParticipant { handle })
                .receive_reply()
                .await?;
            let builtin_publisher = participant.get_builtin_publisher().await?;
            if let Some(spdp_participant_writer) = builtin_publisher
                .lookup_datawriter::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)
                .await?
            {
                let data = deleted_participant
                    .send_actor_mail(domain_participant_actor::AsSpdpDiscoveredParticipantData)
                    .receive_reply()
                    .await;
                spdp_participant_writer.dispose(&data, None).await?;
            }
            deleted_participant.stop().await;
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    /// Async version of [`lookup_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::lookup_participant).
    pub async fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> DdsResult<Option<DomainParticipantAsync>> {
        let domain_participant_list = self
            .domain_participant_factory_actor
            .send_actor_mail(domain_participant_factory_actor::GetParticipantList)
            .receive_reply()
            .await;
        for dp in domain_participant_list {
            if dp
                .send_actor_mail(domain_participant_actor::GetDomainId)?
                .receive_reply()
                .await
                == domain_id
            {
                let status_condition = dp
                    .send_actor_mail(domain_participant_actor::GetStatuscondition)?
                    .receive_reply()
                    .await;
                let builtin_subscriber = dp
                    .send_actor_mail(domain_participant_actor::GetBuiltInSubscriber)?
                    .receive_reply()
                    .await;
                let builtin_subscriber_status_condition_address = builtin_subscriber
                    .send_actor_mail(subscriber_actor::GetStatuscondition)?
                    .receive_reply()
                    .await;
                let timer_handle = dp
                    .send_actor_mail(domain_participant_actor::GetTimerHandle)?
                    .receive_reply()
                    .await;
                let executor_handle = dp
                    .send_actor_mail(domain_participant_actor::GetExecutorHandle)?
                    .receive_reply()
                    .await;
                return Ok(Some(DomainParticipantAsync::new(
                    dp,
                    status_condition,
                    builtin_subscriber,
                    builtin_subscriber_status_condition_address,
                    domain_id,
                    executor_handle,
                    timer_handle,
                )));
            }
        }

        Ok(None)
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
}
