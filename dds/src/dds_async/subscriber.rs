use crate::{
    implementation::{
        actors::{
            data_reader_actor,
            domain_participant_actor::{self, DomainParticipantActor},
            subscriber_actor::{self, SubscriberActor},
        },
        utils::actor::ActorAddress,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        status::{SampleLostStatus, StatusKind},
    },
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
};

use super::{
    condition::StatusConditionAsync, data_reader::DataReaderAsync,
    domain_participant::DomainParticipantAsync, topic::TopicAsync,
};

pub struct SubscriberAsync {
    subscriber_address: ActorAddress<SubscriberActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl SubscriberAsync {
    pub(crate) fn new(
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            subscriber_address,
            participant_address,
            runtime_handle,
        }
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime_handle
    }
}

impl SubscriberAsync {
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datareader<Foo>(
        &self,
        a_topic: &TopicAsync,
        qos: QosKind<DataReaderQos>,
        a_listener: impl DataReaderListener<Foo = Foo> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<DataReaderAsync<Foo>> {
        let type_name = a_topic.get_type_name().await?;
        let topic_name = a_topic.get_name().await?;
        let type_support = self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::get_type_support::new(
                type_name.clone(),
            ))
            .await?
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    type_name
                ))
            })?;

        let listener = Box::new(a_listener);

        let default_unicast_locator_list = self
            .participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::get_default_unicast_locator_list::new(),
            )
            .await?;
        let default_multicast_locator_list = self
            .participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::get_default_unicast_locator_list::new(),
            )
            .await?;

        let has_key = type_support.has_key();

        let reader_address = self
            .subscriber_address
            .send_mail_and_await_reply(subscriber_actor::create_datareader::new(
                type_name,
                topic_name,
                has_key,
                qos,
                listener,
                mask.to_vec(),
                default_unicast_locator_list,
                default_multicast_locator_list,
                self.runtime_handle.clone(),
            ))
            .await??;

        let data_reader = DataReaderAsync::new(
            reader_address,
            self.subscriber_address.clone(),
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        );

        if self
            .subscriber_address
            .send_mail_and_await_reply(subscriber_actor::is_enabled::new())
            .await?
            && self
                .subscriber_address
                .send_mail_and_await_reply(subscriber_actor::get_qos::new())
                .await?
                .entity_factory
                .autoenable_created_entities
        {
            data_reader.enable().await?;
        }

        Ok(data_reader)
    }

    #[tracing::instrument(skip(self, a_datareader))]
    pub async fn delete_datareader<Foo>(
        &self,
        a_datareader: &DataReaderAsync<Foo>,
    ) -> DdsResult<()> {
        let reader_handle = a_datareader.get_instance_handle().await?;
        if self.get_instance_handle().await?
            != a_datareader
                .get_subscriber()
                .await?
                .get_instance_handle()
                .await?
        {
            return Err(DdsError::PreconditionNotMet(
                "Data reader can only be deleted from its parent subscriber".to_string(),
            ));
        }

        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::data_reader_delete::new(reader_handle))
            .await?;

        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::announce_deleted_data_reader::new(
                reader_handle,
            ))
            .await?
    }

    #[tracing::instrument(skip(self))]
    pub async fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataReaderAsync<Foo>>> {
        Ok(self
            .subscriber_address
            .send_mail_and_await_reply(subscriber_actor::lookup_datareader::new(
                topic_name.to_string(),
            ))
            .await?
            .map(|reader_address| {
                DataReaderAsync::new(
                    reader_address,
                    self.subscriber_address.clone(),
                    self.participant_address.clone(),
                    self.runtime_handle.clone(),
                )
            }))
    }

    #[tracing::instrument(skip(self))]
    pub async fn notify_datareaders(&self) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_participant(&self) -> DdsResult<DomainParticipantAsync> {
        Ok(DomainParticipantAsync::new(
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        ))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_default_datareader_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::set_default_datareader_qos::new(qos))
            .await?
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::get_default_datareader_qos::new())
            .await
    }

    #[tracing::instrument]
    pub async fn copy_from_topic_qos(
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, _qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<SubscriberQos> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::get_qos::new())
            .await
    }

    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: impl SubscriberListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusConditionAsync> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::get_statuscondition::new())
            .await
            .map(|c| StatusConditionAsync::new(c, self.runtime_handle.clone()))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        if !self
            .subscriber_address
            .send_mail_and_await_reply(subscriber_actor::is_enabled::new())
            .await?
        {
            self.subscriber_address
                .send_mail_and_await_reply(subscriber_actor::enable::new())
                .await?;

            if self
                .subscriber_address
                .send_mail_and_await_reply(subscriber_actor::get_qos::new())
                .await?
                .entity_factory
                .autoenable_created_entities
            {
                for data_reader in self
                    .subscriber_address
                    .send_mail_and_await_reply(subscriber_actor::data_reader_list::new())
                    .await?
                {
                    data_reader
                        .send_mail_and_await_reply(data_reader_actor::enable::new())
                        .await?;
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::get_instance_handle::new())
            .await
    }
}