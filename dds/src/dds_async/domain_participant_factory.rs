use crate::{
    configuration::DustDdsConfiguration,
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        actors::domain_participant_factory_actor::{self, DomainParticipantFactoryActor},
        utils::actor::Actor,
    },
    infrastructure::{
        error::DdsResult,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};

use super::domain_participant::DomainParticipantAsync;

pub struct DomainParticipantFactoryAsync {
    domain_participant_factory_actor: Actor<DomainParticipantFactoryActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl DomainParticipantFactoryAsync {
    pub fn new(runtime_handle: tokio::runtime::Handle) -> Self {
        let domain_participant_factory_actor =
            Actor::spawn(DomainParticipantFactoryActor::new(), &runtime_handle);

        Self {
            domain_participant_factory_actor,
            runtime_handle,
        }
    }

    pub async fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: impl DomainParticipantListener + Send + 'static,
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

        let domain_participant =
            DomainParticipantAsync::new(participant_address.clone(), self.runtime_handle.clone());

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

    pub async fn delete_participant(&self, participant: &DomainParticipantAsync) -> DdsResult<()> {
        let handle = participant.get_instance_handle().await?;
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::delete_participant::new(
                handle,
            ))
            .await
    }

    pub async fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> DdsResult<Option<DomainParticipantAsync>> {
        Ok(self
            .domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::lookup_participant::new(
                domain_id,
            ))
            .await?
            .map(|dp| DomainParticipantAsync::new(dp, self.runtime_handle.clone())))
    }

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

    pub async fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(
                domain_participant_factory_actor::get_default_participant_qos::new(),
            )
            .await
    }

    pub async fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::set_qos::new(qos))
            .await
    }

    pub async fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::get_qos::new())
            .await
    }

    pub async fn set_configuration(&self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::set_configuration::new(
                configuration,
            ))
            .await
    }

    pub async fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        self.domain_participant_factory_actor
            .send_mail_and_await_reply(domain_participant_factory_actor::get_configuration::new())
            .await
    }
}
