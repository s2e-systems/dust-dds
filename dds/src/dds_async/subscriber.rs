use crate::{
    implementation::{
        actors::{
            data_reader_actor,
            domain_participant_actor::{self, DomainParticipantActor},
            status_condition_actor::StatusConditionActor,
            subscriber_actor::{self, SubscriberActor},
            topic_actor,
        },
        utils::actor::ActorAddress,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        status::{SampleLostStatus, StatusKind},
    },
};

use super::{
    condition::StatusConditionAsync, data_reader::DataReaderAsync,
    data_reader_listener::DataReaderListenerAsync, domain_participant::DomainParticipantAsync,
    subscriber_listener::SubscriberListenerAsync, topic::TopicAsync,
};

/// Async version of [`Subscriber`](crate::subscription::subscriber::Subscriber).
#[derive(Clone)]
pub struct SubscriberAsync {
    subscriber_address: ActorAddress<SubscriberActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    participant: DomainParticipantAsync,
}

impl SubscriberAsync {
    pub(crate) fn new(
        subscriber_address: ActorAddress<SubscriberActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            subscriber_address,
            status_condition_address,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.participant.participant_address()
    }

    pub(crate) fn subscriber_address(&self) -> &ActorAddress<SubscriberActor> {
        &self.subscriber_address
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        self.participant.runtime_handle()
    }
}

impl SubscriberAsync {
    /// Async version of [`create_datareader`](crate::subscription::subscriber::Subscriber::create_datareader).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datareader<Foo>(
        &self,
        a_topic: &TopicAsync,
        qos: QosKind<DataReaderQos>,
        a_listener: impl DataReaderListenerAsync<Foo = Foo> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<DataReaderAsync<Foo>> {
        let type_name = a_topic.get_type_name();
        let topic_name = a_topic.get_name();
        let type_support = self
            .participant_address()
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
            .participant_address()
            .send_mail_and_await_reply(
                domain_participant_actor::get_default_unicast_locator_list::new(),
            )
            .await?;
        let default_multicast_locator_list = self
            .participant_address()
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
                self.runtime_handle().clone(),
                a_topic.topic_address().clone(),
                a_topic.get_statuscondition().address().clone(),
            ))
            .await??;
        let status_condition = reader_address
            .send_mail_and_await_reply(data_reader_actor::get_statuscondition::new())
            .await?;
        let data_reader = DataReaderAsync::new(
            reader_address,
            status_condition,
            self.clone(),
            a_topic.clone(),
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

    /// Async version of [`delete_datareader`](crate::subscription::subscriber::Subscriber::delete_datareader).
    #[tracing::instrument(skip(self, a_datareader))]
    pub async fn delete_datareader<Foo>(
        &self,
        a_datareader: &DataReaderAsync<Foo>,
    ) -> DdsResult<()> {
        let reader_handle = a_datareader.get_instance_handle().await?;
        if self.get_instance_handle().await?
            != a_datareader.get_subscriber().get_instance_handle().await?
        {
            return Err(DdsError::PreconditionNotMet(
                "Data reader can only be deleted from its parent subscriber".to_string(),
            ));
        }

        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::data_reader_delete::new(reader_handle))
            .await?;

        self.participant_address()
            .send_mail_and_await_reply(domain_participant_actor::announce_deleted_data_reader::new(
                reader_handle,
            ))
            .await?
    }

    /// Async version of [`lookup_datareader`](crate::subscription::subscriber::Subscriber::lookup_datareader).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataReaderAsync<Foo>>> {
        if let Some(topic_address) = self
            .participant_address()
            .send_mail_and_await_reply(domain_participant_actor::lookup_topicdescription::new(
                topic_name.to_string(),
            ))
            .await?
        {
            let type_name = topic_address
                .send_mail_and_await_reply(topic_actor::get_type_name::new())
                .await?;
            let topic_status_condition = topic_address
                .send_mail_and_await_reply(topic_actor::get_statuscondition::new())
                .await?;
            let topic = TopicAsync::new(
                topic_address,
                topic_status_condition,
                topic_name.to_string(),
                type_name,
                self.participant.clone(),
            );
            if let Some(dr) = self
                .subscriber_address
                .send_mail_and_await_reply(subscriber_actor::lookup_datareader::new(
                    topic_name.to_string(),
                ))
                .await?
            {
                let status_condition = dr
                    .send_mail_and_await_reply(data_reader_actor::get_statuscondition::new())
                    .await?;
                Ok(Some(DataReaderAsync::new(
                    dr,
                    status_condition,
                    self.clone(),
                    topic,
                )))
            } else {
                Ok(None)
            }
        } else {
            Err(DdsError::BadParameter)
        }
    }

    /// Async version of [`notify_datareaders`](crate::subscription::subscriber::Subscriber::notify_datareaders).
    #[tracing::instrument(skip(self))]
    pub async fn notify_datareaders(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_participant`](crate::subscription::subscriber::Subscriber::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync {
        self.participant.clone()
    }

    /// Async version of [`get_sample_lost_status`](crate::subscription::subscriber::Subscriber::get_sample_lost_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        todo!()
    }

    /// Async version of [`delete_contained_entities`](crate::subscription::subscriber::Subscriber::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`set_default_datareader_qos`](crate::subscription::subscriber::Subscriber::set_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_datareader_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::set_default_datareader_qos::new(qos))
            .await?
    }

    /// Async version of [`get_default_datareader_qos`](crate::subscription::subscriber::Subscriber::get_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::get_default_datareader_qos::new())
            .await
    }

    /// Async version of [`copy_from_topic_qos`](crate::subscription::subscriber::Subscriber::copy_from_topic_qos).
    #[tracing::instrument]
    pub async fn copy_from_topic_qos(
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`set_qos`](crate::subscription::subscriber::Subscriber::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, _qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_qos`](crate::subscription::subscriber::Subscriber::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<SubscriberQos> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::get_qos::new())
            .await
    }

    /// Async version of [`set_listener`](crate::subscription::subscriber::Subscriber::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: impl SubscriberListenerAsync + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle().clone(),
            ))
            .await
    }

    /// Async version of [`get_statuscondition`](crate::subscription::subscriber::Subscriber::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.runtime_handle().clone(),
        )
    }

    /// Async version of [`get_status_changes`](crate::subscription::subscriber::Subscriber::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::subscription::subscriber::Subscriber::enable).
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

    /// Async version of [`get_instance_handle`](crate::subscription::subscriber::Subscriber::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.subscriber_address
            .send_mail_and_await_reply(subscriber_actor::get_instance_handle::new())
            .await
    }
}
