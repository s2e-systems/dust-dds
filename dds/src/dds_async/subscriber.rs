use crate::{
    data_representation_builtin_endpoints::discovered_reader_data::DCPS_SUBSCRIPTION,
    implementation::{
        actor::{Actor, ActorAddress},
        actors::{
            any_data_reader_listener::AnyDataReaderListener,
            data_reader_actor::{self, DataReaderActor},
            domain_participant_actor::{self, DomainParticipantActor},
            status_condition_actor::StatusConditionActor,
            subscriber_actor::{self, SubscriberActor},
            topic_actor::{self, TopicActor},
        },
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

    async fn announce_deleted_data_reader(
        &self,
        reader: &Actor<DataReaderActor>,
        topic: &ActorAddress<TopicActor>,
    ) -> DdsResult<()> {
        let builtin_publisher = self.participant.get_builtin_publisher().await?;
        if let Some(sedp_subscriptions_announcer) = builtin_publisher
            .lookup_datawriter(DCPS_SUBSCRIPTION)
            .await?
        {
            let subscriber_qos = self.get_qos().await?;
            let default_unicast_locator_list = self
                .participant_address()
                .send_actor_mail(domain_participant_actor::GetDefaultUnicastLocatorList)?
                .receive_reply()
                .await;
            let default_multicast_locator_list = self
                .participant_address()
                .send_actor_mail(domain_participant_actor::GetDefaultMulticastLocatorList)?
                .receive_reply()
                .await;
            let topic_data = topic
                .send_actor_mail(topic_actor::GetQos)?
                .receive_reply()
                .await
                .topic_data;
            let xml_type = topic
                .send_actor_mail(topic_actor::GetTypeSupport)?
                .receive_reply()
                .await
                .xml_type();
            let data = reader
                .send_actor_mail(data_reader_actor::AsDiscoveredReaderData {
                    subscriber_qos,
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                    topic_data,
                    xml_type,
                })
                .receive_reply()
                .await?;
            sedp_subscriptions_announcer.dispose(&data, None).await?;
        }
        Ok(())
    }
}

impl SubscriberAsync {
    /// Async version of [`create_datareader`](crate::subscription::subscriber::Subscriber::create_datareader).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datareader<'a, 'b, Foo>(
        &'a self,
        a_topic: &'a TopicAsync,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<(dyn DataReaderListenerAsync<'b, Foo = Foo> + Send + 'b)>>,
        mask: &'a [StatusKind],
    ) -> DdsResult<DataReaderAsync<Foo>>
    where
        Foo: 'b,
    {
        let listener =
            a_listener.map::<Box<dyn AnyDataReaderListener + Send + 'static>, _>(|b| Box::new(b));

        let default_unicast_locator_list = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetDefaultUnicastLocatorList)?
            .receive_reply()
            .await;
        let default_multicast_locator_list = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetDefaultMulticastLocatorList)?
            .receive_reply()
            .await;

        let topic = a_topic.topic_address();
        let topic_name = a_topic.get_name();
        let type_name = a_topic.get_type_name();
        let topic_status_condition = a_topic.get_statuscondition().address().clone();
        let type_support = topic
            .send_actor_mail(topic_actor::GetTypeSupport)?
            .receive_reply()
            .await;
        let has_key = type_support.has_key();

        let reader_address = self
            .subscriber_address
            .send_actor_mail(subscriber_actor::CreateDatareader {
                topic_address: a_topic.topic_address().clone(),
                topic_name,
                type_name,
                topic_status_condition,
                type_support,
                has_key,
                qos,
                a_listener: listener,
                mask: mask.to_vec(),
                default_unicast_locator_list,
                default_multicast_locator_list,
                executor_handle: self.participant.executor_handle().clone(),
            })?
            .receive_reply()
            .await?;

        let status_condition = reader_address
            .send_actor_mail(data_reader_actor::GetStatuscondition)?
            .receive_reply()
            .await;
        let data_reader = DataReaderAsync::new(
            reader_address,
            status_condition,
            self.clone(),
            a_topic.clone(),
        );

        if self
            .subscriber_address
            .send_actor_mail(subscriber_actor::IsEnabled)?
            .receive_reply()
            .await
            && self
                .subscriber_address
                .send_actor_mail(subscriber_actor::GetQos)?
                .receive_reply()
                .await
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

        // Send messages before deleting the reader
        let message_sender_actor = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetMessageSender)?
            .receive_reply()
            .await;
        a_datareader
            .reader_address()
            .send_actor_mail(data_reader_actor::SendMessage {
                message_sender_actor,
            })?;

        let topic = a_datareader.get_topicdescription().topic_address().clone();

        let deleted_reader = self
            .subscriber_address
            .send_actor_mail(subscriber_actor::DeleteDatareader {
                handle: reader_handle,
            })?
            .receive_reply()
            .await?;

        self.announce_deleted_data_reader(&deleted_reader, &topic)
            .await?;
        deleted_reader.stop().await;
        Ok(())
    }

    /// Async version of [`lookup_datareader`](crate::subscription::subscriber::Subscriber::lookup_datareader).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataReaderAsync<Foo>>> {
        if let Some((topic_address, topic_status_condition)) = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::LookupTopicdescription {
                topic_name: topic_name.to_string(),
            })?
            .receive_reply()
            .await?
        {
            let data_reader_list = self
                .subscriber_address
                .send_actor_mail(subscriber_actor::GetDataReaderList)?
                .receive_reply()
                .await;
            for dr in data_reader_list {
                if dr
                    .send_actor_mail(data_reader_actor::GetTopicName)?
                    .receive_reply()
                    .await?
                    == topic_name
                {
                    let type_name = topic_address
                        .send_actor_mail(topic_actor::GetTypeName)?
                        .receive_reply()
                        .await;
                    let topic = TopicAsync::new(
                        topic_address,
                        topic_status_condition,
                        topic_name.to_string(),
                        type_name,
                        self.participant.clone(),
                    );
                    let status_condition = dr
                        .send_actor_mail(data_reader_actor::GetStatuscondition)?
                        .receive_reply()
                        .await;
                    return Ok(Some(DataReaderAsync::new(
                        dr,
                        status_condition,
                        self.clone(),
                        topic,
                    )));
                }
            }
            Ok(None)
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
        let deleted_reader_actor_list = self
            .subscriber_address
            .send_actor_mail(subscriber_actor::DrainDataReaderList)?
            .receive_reply()
            .await;

        let message_sender_actor = self
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetMessageSender)?
            .receive_reply()
            .await;

        for deleted_reader_actor in deleted_reader_actor_list {
            let topic = deleted_reader_actor
                .send_actor_mail(data_reader_actor::GetTopicAddress)
                .receive_reply()
                .await;

            // Send messages before deleting the reader
            deleted_reader_actor.send_actor_mail(data_reader_actor::SendMessage {
                message_sender_actor: message_sender_actor.clone(),
            });

            self.announce_deleted_data_reader(&deleted_reader_actor, &topic)
                .await?;
            deleted_reader_actor.stop().await;
        }
        Ok(())
    }

    /// Async version of [`set_default_datareader_qos`](crate::subscription::subscriber::Subscriber::set_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_datareader_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        self.subscriber_address
            .send_actor_mail(subscriber_actor::SetDefaultDatareaderQos { qos })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_default_datareader_qos`](crate::subscription::subscriber::Subscriber::get_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        Ok(self
            .subscriber_address
            .send_actor_mail(subscriber_actor::GetDefaultDatareaderQos)?
            .receive_reply()
            .await)
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
        Ok(self
            .subscriber_address
            .send_actor_mail(subscriber_actor::GetQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`set_listener`](crate::subscription::subscriber::Subscriber::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.subscriber_address
            .send_actor_mail(subscriber_actor::SetListener {
                listener: a_listener,
                status_kind: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_statuscondition`](crate::subscription::subscriber::Subscriber::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.participant.executor_handle().clone(),
            self.participant.timer_handle().clone(),
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
            .send_actor_mail(subscriber_actor::IsEnabled)?
            .receive_reply()
            .await
        {
            self.subscriber_address
                .send_actor_mail(subscriber_actor::Enable)?
                .receive_reply()
                .await;

            if self
                .subscriber_address
                .send_actor_mail(subscriber_actor::GetQos)?
                .receive_reply()
                .await
                .entity_factory
                .autoenable_created_entities
            {
                for data_reader in self
                    .subscriber_address
                    .send_actor_mail(subscriber_actor::GetDataReaderList)?
                    .receive_reply()
                    .await
                {
                    data_reader
                        .send_actor_mail(data_reader_actor::Enable)?
                        .receive_reply()
                        .await;
                }
            }
        }

        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::subscription::subscriber::Subscriber::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self
            .subscriber_address
            .send_actor_mail(subscriber_actor::GetInstanceHandle)?
            .receive_reply()
            .await)
    }
}
