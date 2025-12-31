use super::{
    condition::StatusConditionAsync, data_reader::DataReaderAsync,
    data_reader_listener::DataReaderListener, domain_participant::DomainParticipantAsync,
    dynamic_data_reader::DynamicDataReaderAsync, subscriber_listener::SubscriberListener,
};
use crate::{
    dcps::{
        actor::ActorAddress,
        channels::{mpsc::MpscSender, oneshot::oneshot},
        domain_participant_mail::{DcpsDomainParticipantMail, SubscriberServiceMail},
        listeners::{
            data_reader_listener::DcpsDataReaderListener,
            subscriber_listener::DcpsSubscriberListener,
        },
        status_condition::DcpsStatusCondition,
    },
    dds_async::topic_description::TopicDescriptionAsync,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        status::{SampleLostStatus, StatusKind},
    },
    xtypes::dynamic_type::DynamicType,
};
use alloc::{string::String, vec::Vec};

/// Async version of [`Subscriber`](crate::subscription::subscriber::Subscriber).
pub struct SubscriberAsync {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<DcpsStatusCondition>,
    participant: DomainParticipantAsync,
}

impl Clone for SubscriberAsync {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            status_condition_address: self.status_condition_address.clone(),
            participant: self.participant.clone(),
        }
    }
}

impl SubscriberAsync {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<DcpsStatusCondition>,
        participant: DomainParticipantAsync,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            participant,
        }
    }

    pub(crate) fn participant_address(&self) -> &MpscSender<DcpsDomainParticipantMail> {
        self.participant.participant_address()
    }
}

impl SubscriberAsync {
    /// Async version of [`create_datareader`](crate::subscription::subscriber::Subscriber::create_datareader).
    #[tracing::instrument(skip(self, a_topic, a_listener))]
    pub async fn create_datareader<Foo>(
        &self,
        a_topic: &TopicDescriptionAsync,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<impl DataReaderListener<Foo> + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<DataReaderAsync<Foo>> {
        let dcps_listener = a_listener.map(DcpsDataReaderListener::new);
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Subscriber(
                SubscriberServiceMail::CreateDataReader {
                    subscriber_handle: self.handle,
                    topic_name: a_topic.get_name(),
                    qos,
                    dcps_listener,
                    mask: mask.to_vec(),
                    domain_participant_address: self.participant_address().clone(),
                    reply_sender,
                },
            ))
            .await?;
        let (guid, reader_status_condition_address) = reply_receiver.await??;

        Ok(DataReaderAsync::new(
            guid,
            reader_status_condition_address,
            self.clone(),
            a_topic.clone(),
        ))
    }

    /// This operation creates a [`DynamicDataReaderAsync`] for the given topic name and [`DynamicType`].
    /// Unlike [`create_datareader`](Self::create_datareader), this operation does not require a [`TopicAsync`] and accepts a
    /// [`DynamicType`] directly, allowing readers to be created for types discovered at runtime via the TypeLookup service.
    /// The returned [`DynamicDataReaderAsync`] will be attached and belong to the [`SubscriberAsync`].
    #[tracing::instrument(skip(self, dynamic_type))]
    pub async fn create_dynamic_datareader(
        &self,
        topic_name: &str,
        dynamic_type: DynamicType,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<DynamicDataReaderAsync> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Subscriber(
                SubscriberServiceMail::CreateDynamicDataReader {
                    subscriber_handle: self.handle,
                    topic_name: String::from(topic_name),
                    dynamic_type: dynamic_type.clone(),
                    qos,
                    domain_participant_address: self.participant_address().clone(),
                    reply_sender,
                },
            ))
            .await?;
        let (handle, status_condition_address) = reply_receiver.await??;

        Ok(DynamicDataReaderAsync::new(
            handle,
            status_condition_address,
            self.clone(),
            String::from(topic_name),
            dynamic_type,
        ))
    }

    /// Async version of [`delete_datareader`](crate::subscription::subscriber::Subscriber::delete_datareader).
    #[tracing::instrument(skip(self, a_datareader))]
    pub async fn delete_datareader<Foo>(
        &self,
        a_datareader: &DataReaderAsync<Foo>,
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Subscriber(
                SubscriberServiceMail::DeleteDataReader {
                    subscriber_handle: self.handle,
                    datareader_handle: a_datareader.get_instance_handle().await,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`lookup_datareader`](crate::subscription::subscriber::Subscriber::lookup_datareader).
    #[tracing::instrument(skip(self))]
    pub async fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<DataReaderAsync<Foo>>> {
        if let Some(topic) = self.participant.lookup_topicdescription(topic_name).await? {
            let (reply_sender, reply_receiver) = oneshot();
            self.participant_address()
                .send(DcpsDomainParticipantMail::Subscriber(
                    SubscriberServiceMail::LookupDataReader {
                        subscriber_handle: self.handle,
                        topic_name: String::from(topic_name),
                        reply_sender,
                    },
                ))
                .await?;
            if let Some((reader_handle, reader_status_condition_address)) = reply_receiver.await?? {
                Ok(Some(DataReaderAsync::new(
                    reader_handle,
                    reader_status_condition_address,
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
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Subscriber(
                SubscriberServiceMail::SetDefaultDataReaderQos {
                    subscriber_handle: self.handle,
                    qos,
                    reply_sender,
                },
            ))
            .await?;

        reply_receiver.await?
    }

    /// Async version of [`get_default_datareader_qos`](crate::subscription::subscriber::Subscriber::get_default_datareader_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Subscriber(
                SubscriberServiceMail::GetDefaultDataReaderQos {
                    subscriber_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
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
    pub async fn set_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Subscriber(
                SubscriberServiceMail::SetQos {
                    subscriber_handle: self.handle,
                    qos,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_qos`](crate::subscription::subscriber::Subscriber::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<SubscriberQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Subscriber(
                SubscriberServiceMail::GetSubscriberQos {
                    subscriber_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`set_listener`](crate::subscription::subscriber::Subscriber::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<impl SubscriberListener + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let dcps_listener = a_listener.map(|l| DcpsSubscriberListener::new(l));
        self.participant_address()
            .send(DcpsDomainParticipantMail::Subscriber(
                SubscriberServiceMail::SetListener {
                    subscriber_handle: self.handle,
                    dcps_listener,
                    mask: mask.to_vec(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_statuscondition`](crate::subscription::subscriber::Subscriber::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.status_condition_address.clone())
    }

    /// Async version of [`get_status_changes`](crate::subscription::subscriber::Subscriber::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::subscription::subscriber::Subscriber::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_instance_handle`](crate::subscription::subscriber::Subscriber::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
