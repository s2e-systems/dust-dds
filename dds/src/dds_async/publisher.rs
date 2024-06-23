use crate::{
    data_representation_builtin_endpoints::discovered_writer_data::DCPS_PUBLICATION,
    implementation::{
        actor::{Actor, ActorAddress},
        actors::{
            any_data_writer_listener::AnyDataWriterListener,
            data_writer_actor::{self, DataWriterActor},
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor::{self, PublisherActor},
            status_condition_actor::StatusConditionActor,
            topic_actor::{self, TopicActor},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
};

use super::{
    condition::StatusConditionAsync, data_writer::DataWriterAsync,
    data_writer_listener::DataWriterListenerAsync, domain_participant::DomainParticipantAsync,
    publisher_listener::PublisherListenerAsync, topic::TopicAsync,
};

/// Async version of [`Publisher`](crate::publication::publisher::Publisher).
#[derive(Clone)]
pub struct PublisherAsync {
    publisher_address: ActorAddress<PublisherActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    participant: DomainParticipantAsync,
}

impl PublisherAsync {
    pub(crate) fn new(
        publisher_address: ActorAddress<PublisherActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            publisher_address,
            status_condition_address,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.participant.participant_address()
    }

    pub(crate) fn publisher_address(&self) -> &ActorAddress<PublisherActor> {
        &self.publisher_address
    }

    async fn announce_deleted_data_writer(
        &self,
        writer: &Actor<DataWriterActor>,
        topic: &ActorAddress<TopicActor>,
    ) -> DdsResult<()> {
        let builtin_publisher = self.participant.get_builtin_publisher().await?;
        if let Some(sedp_publications_announcer) = builtin_publisher
            .lookup_datawriter(DCPS_PUBLICATION)
            .await?
        {
            let publisher_qos = self.get_qos().await?;
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
            let data = writer
                .send_actor_mail(data_writer_actor::AsDiscoveredWriterData {
                    publisher_qos,
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                    topic_data,
                    xml_type,
                })
                .receive_reply()
                .await?;
            sedp_publications_announcer.dispose(&data, None).await?;
        }
        Ok(())
    }
}

impl PublisherAsync {
    /// Async version of [`create_datawriter`](crate::publication::publisher::Publisher::create_datawriter).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datawriter<'a, 'b, Foo>(
        &'a self,
        a_topic: &'a TopicAsync,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListenerAsync<'b, Foo = Foo> + Send + 'b>>,
        mask: &'a [StatusKind],
    ) -> DdsResult<DataWriterAsync<Foo>>
    where
        Foo: 'b,
    {
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
        let data_max_size_serialized = self
            .participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetDataMaxSizeSerialized)?
            .receive_reply()
            .await;

        let listener = a_listener.map::<Box<dyn AnyDataWriterListener + Send>, _>(|b| Box::new(b));
        let has_key = a_topic
            .topic_address()
            .send_actor_mail(topic_actor::GetTypeSupport)?
            .receive_reply()
            .await
            .has_key();
        let topic_name = a_topic.get_name();
        let type_name = a_topic.get_type_name();
        let topic_status_condition = a_topic.get_statuscondition().address().clone();
        let data_writer_address = self
            .publisher_address
            .send_actor_mail(publisher_actor::CreateDatawriter {
                topic_address: a_topic.topic_address().clone(),
                topic_name,
                type_name,
                topic_status_condition,
                has_key,
                data_max_size_serialized,
                qos,
                a_listener: listener,
                mask: mask.to_vec(),
                default_unicast_locator_list,
                default_multicast_locator_list,
                executor_handle: self.participant.executor_handle().clone(),
            })?
            .receive_reply()
            .await?;
        let status_condition = data_writer_address
            .send_actor_mail(data_writer_actor::GetStatuscondition)?
            .receive_reply()
            .await;
        let data_writer = DataWriterAsync::new(
            data_writer_address,
            status_condition,
            self.clone(),
            a_topic.clone(),
        );

        if self
            .publisher_address
            .send_actor_mail(publisher_actor::IsEnabled)?
            .receive_reply()
            .await
            && self
                .publisher_address
                .send_actor_mail(publisher_actor::GetQos)?
                .receive_reply()
                .await
                .entity_factory
                .autoenable_created_entities
        {
            data_writer.enable().await?
        }

        Ok(data_writer)
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::delete_datawriter).
    #[tracing::instrument(skip(self, a_datawriter))]
    pub async fn delete_datawriter<Foo>(
        &self,
        a_datawriter: &DataWriterAsync<Foo>,
    ) -> DdsResult<()> {
        let writer_handle = a_datawriter.get_instance_handle().await?;

        let message_sender_actor = self
            .participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetMessageSender)?
            .receive_reply()
            .await;

        a_datawriter
            .writer_address()
            .send_actor_mail(data_writer_actor::SendMessage {
                message_sender_actor,
            })?;
        let topic = a_datawriter.get_topic();

        let deleted_writer = self
            .publisher_address
            .send_actor_mail(publisher_actor::DeleteDatawriter {
                handle: writer_handle,
            })?
            .receive_reply()
            .await?;

        self.announce_deleted_data_writer(&deleted_writer, topic.topic_address())
            .await?;
        deleted_writer.stop().await;
        Ok(())
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::lookup_datawriter).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datawriter<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataWriterAsync<Foo>>> {
        if let Some((topic_address, topic_status_condition)) = self
            .participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::LookupTopicdescription {
                topic_name: topic_name.to_string(),
            })?
            .receive_reply()
            .await?
        {
            let data_writer_list = self
                .publisher_address
                .send_actor_mail(publisher_actor::GetDataWriterList)?
                .receive_reply()
                .await;
            for dw in data_writer_list {
                if dw
                    .send_actor_mail(data_writer_actor::GetTopicName)?
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
                    let status_condition = dw
                        .send_actor_mail(data_writer_actor::GetStatuscondition)?
                        .receive_reply()
                        .await;
                    return Ok(Some(DataWriterAsync::new(
                        dw.clone(),
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

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::suspend_publications).
    #[tracing::instrument(skip(self))]
    pub async fn suspend_publications(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::resume_publications).
    #[tracing::instrument(skip(self))]
    pub async fn resume_publications(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::begin_coherent_changes).
    #[tracing::instrument(skip(self))]
    pub async fn begin_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::end_coherent_changes).
    #[tracing::instrument(skip(self))]
    pub async fn end_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`delete_datawriter`](crate::publication::publisher::Publisher::wait_for_acknowledgments).
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_participant`](crate::publication::publisher::Publisher::get_participant).
    #[tracing::instrument(skip(self))]
    pub fn get_participant(&self) -> DomainParticipantAsync {
        self.participant.clone()
    }

    /// Async version of [`delete_contained_entities`](crate::publication::publisher::Publisher::delete_contained_entities).
    #[tracing::instrument(skip(self))]
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        let deleted_writer_actor_list = self
            .publisher_address
            .send_actor_mail(publisher_actor::DrainDataWriterList)?
            .receive_reply()
            .await;
        let message_sender_actor = self
            .participant
            .participant_address()
            .send_actor_mail(domain_participant_actor::GetMessageSender)?
            .receive_reply()
            .await;

        for deleted_writer_actor in deleted_writer_actor_list {
            let topic_address = deleted_writer_actor
                .send_actor_mail(data_writer_actor::GetTopicAddress)
                .receive_reply()
                .await;
            deleted_writer_actor.send_actor_mail(data_writer_actor::SendMessage {
                message_sender_actor: message_sender_actor.clone(),
            });

            self.announce_deleted_data_writer(&deleted_writer_actor, &topic_address)
                .await?;
            deleted_writer_actor.stop().await;
        }
        Ok(())
    }

    /// Async version of [`set_default_datawriter_qos`](crate::publication::publisher::Publisher::set_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                self.publisher_address
                    .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
                    .receive_reply()
                    .await
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.publisher_address
            .send_actor_mail(publisher_actor::SetDefaultDatawriterQos { qos })?
            .receive_reply()
            .await;

        Ok(())
    }

    /// Async version of [`get_default_datawriter_qos`](crate::publication::publisher::Publisher::get_default_datawriter_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        Ok(self
            .publisher_address
            .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`copy_from_topic_qos`](crate::publication::publisher::Publisher::copy_from_topic_qos).
    #[tracing::instrument(skip(self))]
    pub async fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }
}

impl PublisherAsync {
    /// Async version of [`set_qos`](crate::publication::publisher::Publisher::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, _qos: QosKind<PublisherQos>) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_qos`](crate::publication::publisher::Publisher::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<PublisherQos> {
        Ok(self
            .publisher_address
            .send_actor_mail(publisher_actor::GetQos)?
            .receive_reply()
            .await)
    }

    /// Async version of [`set_listener`](crate::publication::publisher::Publisher::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.publisher_address
            .send_actor_mail(publisher_actor::SetListener {
                listener: a_listener,
                status_kind: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_statuscondition`](crate::publication::publisher::Publisher::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(
            self.status_condition_address.clone(),
            self.participant.executor_handle().clone(),
            self.participant.timer_handle().clone(),
        )
    }

    /// Async version of [`get_status_changes`](crate::publication::publisher::Publisher::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::publication::publisher::Publisher::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.publisher_address
            .send_actor_mail(publisher_actor::Enable)?
            .receive_reply()
            .await;
        Ok(())
    }

    /// Async version of [`get_instance_handle`](crate::publication::publisher::Publisher::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self
            .publisher_address
            .send_actor_mail(publisher_actor::GetInstanceHandle)?
            .receive_reply()
            .await)
    }
}
