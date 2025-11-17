use tracing::warn;

use super::{condition::StatusConditionAsync, publisher::PublisherAsync};
use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps::{
        actor::ActorAddress,
        channels::{mpsc::MpscSender, oneshot::oneshot},
        domain_participant_mail::{
            DcpsDomainParticipantMail, MessageServiceMail, WriterServiceMail,
        },
        listeners::data_writer_listener::DcpsDataWriterListener,
        status_condition::DcpsStatusCondition,
    },
    dds_async::topic_description::TopicDescriptionAsync,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::Time,
        type_support::TypeSupport,
    },
    publication::data_writer_listener::DataWriterListener,
    runtime::DdsRuntime,
};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Async version of [`DataWriter`](crate::publication::data_writer::DataWriter).
pub struct DataWriterAsync<R: DdsRuntime, Foo> {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<DcpsStatusCondition>,
    publisher: PublisherAsync<R>,
    topic: TopicDescriptionAsync<R>,
    phantom: PhantomData<Foo>,
}

impl<R: DdsRuntime, Foo> Clone for DataWriterAsync<R, Foo> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            status_condition_address: self.status_condition_address.clone(),
            publisher: self.publisher.clone(),
            topic: self.topic.clone(),
            phantom: self.phantom,
        }
    }
}

impl<R: DdsRuntime, Foo> DataWriterAsync<R, Foo> {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<DcpsStatusCondition>,
        publisher: PublisherAsync<R>,
        topic: TopicDescriptionAsync<R>,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            publisher,
            topic,
            phantom: PhantomData,
        }
    }

    pub(crate) fn participant_address(&self) -> &MpscSender<DcpsDomainParticipantMail<R>> {
        self.publisher.participant_address()
    }

    pub(crate) fn change_foo_type<T>(self) -> DataWriterAsync<R, T> {
        DataWriterAsync {
            handle: self.handle,
            status_condition_address: self.status_condition_address,
            publisher: self.publisher,
            topic: self.topic,
            phantom: PhantomData,
        }
    }
}

impl<R: DdsRuntime, Foo> DataWriterAsync<R, Foo>
where
    Foo: TypeSupport,
{
    /// Async version of [`register_instance`](crate::publication::data_writer::DataWriter::register_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn register_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        let timestamp = self
            .get_publisher()
            .get_participant()
            .get_current_time()
            .await?;
        self.register_instance_w_timestamp(instance, timestamp)
            .await
    }

    /// Async version of [`register_instance_w_timestamp`](crate::publication::data_writer::DataWriter::register_instance_w_timestamp).
    #[tracing::instrument(skip(self, _instance))]
    pub async fn register_instance_w_timestamp(
        &self,
        _instance: &Foo,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    /// Async version of [`unregister_instance`](crate::publication::data_writer::DataWriter::unregister_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn unregister_instance(
        &self,
        instance: Foo,
        handle: Option<InstanceHandle>,
    ) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()
            .get_participant()
            .get_current_time()
            .await?;
        self.unregister_instance_w_timestamp(instance, handle, timestamp)
            .await
    }

    /// Async version of [`unregister_instance_w_timestamp`](crate::publication::data_writer::DataWriter::unregister_instance_w_timestamp).
    #[tracing::instrument(skip(self, instance))]
    pub async fn unregister_instance_w_timestamp(
        &self,
        instance: Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let dynamic_data = instance.create_dynamic_sample();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::UnregisterInstance {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    dynamic_data,
                    timestamp,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_key_value`](crate::publication::data_writer::DataWriter::get_key_value).
    #[tracing::instrument(skip(self, _key_holder))]
    pub async fn get_key_value(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`lookup_instance`](crate::publication::data_writer::DataWriter::lookup_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn lookup_instance(&self, instance: Foo) -> DdsResult<Option<InstanceHandle>> {
        let (reply_sender, reply_receiver) = oneshot();
        let dynamic_data = instance.create_dynamic_sample();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::LookupInstance {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    dynamic_data,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`write`](crate::publication::data_writer::DataWriter::write).
    #[tracing::instrument(skip(self, data))]
    pub async fn write(&self, data: Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()
            .get_participant()
            .get_current_time()
            .await?;
        self.write_w_timestamp(data, handle, timestamp).await
    }

    /// Async version of [`write_w_timestamp`](crate::publication::data_writer::DataWriter::write_w_timestamp).
    #[tracing::instrument(skip(self, data))]
    pub async fn write_w_timestamp(
        &self,
        data: Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let dynamic_data = data.create_dynamic_sample();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::WriteWTimestamp {
                    participant_address: self.participant_address().clone(),
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    dynamic_data,
                    timestamp,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`dispose`](crate::publication::data_writer::DataWriter::dispose).
    #[tracing::instrument(skip(self, data))]
    pub async fn dispose(&self, data: Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()
            .get_participant()
            .get_current_time()
            .await?;
        self.dispose_w_timestamp(data, handle, timestamp).await
    }

    /// Async version of [`dispose_w_timestamp`](crate::publication::data_writer::DataWriter::dispose_w_timestamp).
    #[tracing::instrument(skip(self, data))]
    pub async fn dispose_w_timestamp(
        &self,
        data: Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let dynamic_data = data.create_dynamic_sample();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::DisposeWTimestamp {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    dynamic_data,
                    timestamp,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }
}

impl<R: DdsRuntime, Foo> DataWriterAsync<R, Foo> {
    /// Async version of [`wait_for_acknowledgments`](crate::publication::data_writer::DataWriter::wait_for_acknowledgments).
    /// This method does not internally wait for a maximum timeout and that is expected
    /// to be handle on the user side if needed.
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_acknowledgments(&self) -> DdsResult<()> {
        let participant_address = self.participant_address().clone();
        let publisher_handle = self.get_publisher().get_instance_handle().await;
        let data_writer_handle = self.handle;
        loop {
            let (reply_sender, reply_receiver) = oneshot();
            participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AreAllChangesAcknowledged {
                        publisher_handle,
                        data_writer_handle,
                        reply_sender,
                    },
                ))
                .await
                .ok();
            let reply = reply_receiver.await;
            match reply {
                Ok(are_changes_acknowledged) => match are_changes_acknowledged {
                    Ok(true) => return Ok(()),
                    Ok(false) => (),
                    Err(e) => return Err(e),
                },
                Err(_) => return Err(DdsError::Error(String::from("Channel error"))),
            }
        }
    }

    /// Async version of [`get_liveliness_lost_status`](crate::publication::data_writer::DataWriter::get_liveliness_lost_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        todo!()
    }

    /// Async version of [`get_offered_deadline_missed_status`](crate::publication::data_writer::DataWriter::get_offered_deadline_missed_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_offered_deadline_missed_status(
        &self,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::GetOfferedDeadlineMissedStatus {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_offered_incompatible_qos_status`](crate::publication::data_writer::DataWriter::get_offered_incompatible_qos_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_offered_incompatible_qos_status(
        &self,
    ) -> DdsResult<OfferedIncompatibleQosStatus> {
        todo!()
    }

    /// Async version of [`get_publication_matched_status`](crate::publication::data_writer::DataWriter::get_publication_matched_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::GetPublicationMatchedStatus {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_topic`](crate::publication::data_writer::DataWriter::get_topic).
    #[tracing::instrument(skip(self))]
    pub fn get_topic(&self) -> TopicDescriptionAsync<R> {
        self.topic.clone()
    }

    /// Async version of [`get_publisher`](crate::publication::data_writer::DataWriter::get_publisher).
    #[tracing::instrument(skip(self))]
    pub fn get_publisher(&self) -> PublisherAsync<R> {
        self.publisher.clone()
    }

    /// Async version of [`assert_liveliness`](crate::publication::data_writer::DataWriter::assert_liveliness).
    #[tracing::instrument(skip(self))]
    pub async fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_matched_subscription_data`](crate::publication::data_writer::DataWriter::get_matched_subscription_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::GetMatchedSubscriptionData {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    subscription_handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_matched_subscriptions`](crate::publication::data_writer::DataWriter::get_matched_subscriptions).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::GetMatchedSubscriptions {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }
}

impl<R: DdsRuntime, Foo> DataWriterAsync<R, Foo> {
    /// Async version of [`set_qos`](crate::publication::data_writer::DataWriter::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::SetDataWriterQos {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    qos,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_qos`](crate::publication::data_writer::DataWriter::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataWriterQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::GetDataWriterQos {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_statuscondition`](crate::publication::data_writer::DataWriter::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.status_condition_address.clone())
    }

    /// Async version of [`get_status_changes`](crate::publication::data_writer::DataWriter::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::publication::data_writer::DataWriter::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::EnableDataWriter {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    participant_address: self.participant_address().clone(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }

    /// Async version of [`get_instance_handle`](crate::publication::data_writer::DataWriter::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
impl<R: DdsRuntime, Foo> DataWriterAsync<R, Foo> {
    /// Async version of [`set_listener`](crate::publication::data_writer::DataWriter::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<impl DataWriterListener<R, Foo> + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let dcps_listener = a_listener.map(DcpsDataWriterListener::new);
        self.participant_address()
            .send(DcpsDomainParticipantMail::Writer(
                WriterServiceMail::SetListener {
                    publisher_handle: self.publisher.get_instance_handle().await,
                    data_writer_handle: self.handle,
                    dcps_listener,
                    listener_mask: mask.to_vec(),
                    reply_sender,
                },
            ))
            .await?;
        reply_receiver.await?
    }
}
