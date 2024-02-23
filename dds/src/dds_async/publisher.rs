use crate::{
    implementation::{
        actors::{
            domain_participant_actor::{self, DomainParticipantActor},
            publisher_actor::{self, PublisherActor},
        },
        utils::actor::ActorAddress,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::Duration,
    },
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
};

use super::{
    condition::StatusConditionAsync, data_writer::DataWriterAsync, domain_participant::DomainParticipantAsync, topic::TopicAsync
};

pub struct PublisherAsync {
    publisher_address: ActorAddress<PublisherActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl PublisherAsync {
    pub(crate) fn new(
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            publisher_address,
            participant_address,
            runtime_handle,
        }
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime_handle
    }
}

impl PublisherAsync {
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datawriter<Foo>(
        &self,
        a_topic: &TopicAsync,
        qos: QosKind<DataWriterQos>,
        a_listener: impl DataWriterListener<Foo = Foo> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<DataWriterAsync<Foo>> {
        let type_name = a_topic.get_type_name().await?;
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

        let default_unicast_locator_list = self
            .participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::get_default_unicast_locator_list::new(),
            )
            .await?;
        let default_multicast_locator_list = self
            .participant_address
            .send_mail_and_await_reply(
                domain_participant_actor::get_default_multicast_locator_list::new(),
            )
            .await?;
        let data_max_size_serialized = self
            .participant_address
            .send_mail_and_await_reply(domain_participant_actor::data_max_size_serialized::new())
            .await?;

        let listener = Box::new(a_listener);
        let has_key = type_support.has_key();
        let data_writer_address = self
            .publisher_address
            .send_mail_and_await_reply(publisher_actor::create_datawriter::new(
                a_topic.get_type_name().await?,
                a_topic.get_name().await?,
                has_key,
                data_max_size_serialized,
                qos,
                listener,
                mask.to_vec(),
                default_unicast_locator_list,
                default_multicast_locator_list,
                self.runtime_handle.clone(),
            ))
            .await??;

        let data_writer = DataWriterAsync::new(
            data_writer_address,
            self.publisher_address.clone(),
            self.participant_address.clone(),
            self.runtime_handle.clone(),
        );

        if self
            .publisher_address
            .send_mail_and_await_reply(publisher_actor::is_enabled::new())
            .await?
            && self
                .publisher_address
                .send_mail_and_await_reply(publisher_actor::get_qos::new())
                .await?
                .entity_factory
                .autoenable_created_entities
        {
            data_writer.enable().await?
        }

        Ok(data_writer)
    }

    #[tracing::instrument(skip(self, a_datawriter))]
    pub async fn delete_datawriter<Foo>(
        &self,
        a_datawriter: &DataWriterAsync<Foo>,
    ) -> DdsResult<()> {
        let writer_handle = a_datawriter.get_instance_handle().await?;
        if self
            .publisher_address
            .send_mail_and_await_reply(publisher_actor::get_instance_handle::new())
            .await?
            != a_datawriter
                .get_publisher()
                .await?
                .get_instance_handle()
                .await?
        {
            return Err(DdsError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ));
        }

        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::datawriter_delete::new(writer_handle))
            .await?;

        self.participant_address
            .send_mail_and_await_reply(domain_participant_actor::announce_deleted_data_writer::new(
                writer_handle,
            ))
            .await?
    }

    #[tracing::instrument(skip(self))]
    pub async fn lookup_datawriter<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataWriterAsync<Foo>>> {
        Ok(self
            .publisher_address
            .send_mail_and_await_reply(publisher_actor::lookup_datawriter::new(
                topic_name.to_string(),
            ))
            .await?
            .map(|dw| {
                DataWriterAsync::new(
                    dw,
                    self.publisher_address.clone(),
                    self.participant_address.clone(),
                    self.runtime_handle.clone(),
                )
            }))
    }

    #[tracing::instrument(skip(self))]
    pub async fn suspend_publications(&self) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn resume_publications(&self) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn begin_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn end_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
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
    pub async fn delete_contained_entities(&self) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => {
                self.publisher_address
                    .send_mail_and_await_reply(publisher_actor::get_default_datawriter_qos::new())
                    .await?
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::set_default_datawriter_qos::new(qos))
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::get_default_datawriter_qos::new())
            .await
    }

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
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, _qos: QosKind<PublisherQos>) -> DdsResult<()> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<PublisherQos> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::get_qos::new())
            .await
    }

    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: impl PublisherListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::set_listener::new(
                Box::new(a_listener),
                mask.to_vec(),
                self.runtime_handle.clone(),
            ))
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_statuscondition(&self) -> DdsResult<StatusConditionAsync> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::enable::new())
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.publisher_address
            .send_mail_and_await_reply(publisher_actor::get_instance_handle::new())
            .await
    }
}
