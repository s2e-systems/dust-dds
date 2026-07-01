use alloc::{string::String, vec::Vec};

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps::{
        channels::oneshot::{OneshotSender, oneshot},
        dcps_domain_participant::{DcpsDomainParticipant, RtpsWriterKind, serialize},
        dcps_mail::{DcpsMail, EventServiceMail, MessageServiceMail, WriterServiceMail},
        listeners::data_writer_listener::DcpsDataWriterListener,
        status_mask::StatusMask,
        xtypes_glue::key_and_instance_handle::{
            KeyHolderData, get_instance_handle_from_key_holder_data,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        qos_policy::{HistoryQosPolicyKind, ReliabilityQosPolicyKind},
        status::{OfferedDeadlineMissedStatus, PublicationMatchedStatus, StatusKind},
        time::{Duration, DurationKind, Time},
    },
    runtime::{Clock, DdsRuntime, Either, Spawner, Timer, select_future},
    xtypes::dynamic_type::DynamicData,
};

impl DcpsDomainParticipant {
    #[tracing::instrument(skip(self))]
    pub fn get_publication_matched_status(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
    ) -> DdsResult<PublicationMatchedStatus> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let status = data_writer.get_publication_matched_status();

        data_writer
            .status_condition
            .remove_communication_state(StatusKind::PublicationMatched);
        Ok(status)
    }

    #[tracing::instrument(skip(self, dcps_listener, runtime))]
    pub fn set_listener_data_writer(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        dcps_listener: Option<DcpsDataWriterListener>,
        listener_mask: StatusMask,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Ok(());
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Ok(());
        };

        let listener_sender = dcps_listener.map(|l| l.spawn(&runtime.spawner()));
        data_writer.listener_sender = listener_sender;
        data_writer.listener_mask = listener_mask;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_data_writer_qos(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_subscriptions(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_writer
            .matched_subscription_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_subscription_data(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        subscription_handle: &InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_writer
            .matched_subscription_list
            .iter()
            .find(|x| subscription_handle.as_ref() == &x.key().value)
            .ok_or(DdsError::BadParameter)
            .cloned()
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn unregister_instance(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        dynamic_data: &DynamicData<'static>,
        timestamp: Time,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_writer.unregister_w_timestamp(
            dynamic_data,
            timestamp,
            self.transport.message_writer.as_ref(),
            runtime,
        )
    }

    #[tracing::instrument(skip(self))]
    pub fn lookup_instance(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        dynamic_data: &DynamicData<'static>,
    ) -> DdsResult<Option<InstanceHandle>> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !data_writer.enabled {
            return Err(DdsError::NotEnabled);
        }

        let key_holder_data = match KeyHolderData::from_dynamic_data(dynamic_data) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };
        let instance_handle = match get_instance_handle_from_key_holder_data(&key_holder_data) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(data_writer
            .registered_instance_list
            .contains(&instance_handle)
            .then_some(instance_handle))
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, reply_sender, runtime))]
    pub fn write_w_timestamp(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        dynamic_data: &DynamicData<'static>,
        timestamp: Time,
        runtime: &impl DdsRuntime,
        reply_sender: OneshotSender<DdsResult<()>>,
    ) {
        let now = runtime.clock().now();
        let Some(publisher) = core::iter::once(&mut self.domain_participant.builtin_publisher)
            .chain(&mut self.domain_participant.user_defined_publisher_list)
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        if !data_writer.enabled {
            reply_sender.send(Err(DdsError::NotEnabled));
            return;
        }

        let serialized_data = match serialize(dynamic_data, &data_writer.qos.representation) {
            Ok(s) => s,
            Err(e) => {
                reply_sender.send(Err(e));
                return;
            }
        };

        let key_holder_data = match KeyHolderData::from_dynamic_data(dynamic_data) {
            Ok(h) => h,
            Err(e) => {
                reply_sender.send(Err(e.into()));
                return;
            }
        };
        let instance_handle = match get_instance_handle_from_key_holder_data(&key_holder_data) {
            Ok(h) => h,
            Err(e) => {
                reply_sender.send(Err(e.into()));
                return;
            }
        };

        if let HistoryQosPolicyKind::KeepLast(depth) = data_writer.qos.history.kind {
            if let Some(s) = data_writer
                .instance_samples
                .iter_mut()
                .find(|x| x.instance == instance_handle)
            {
                if s.samples.len() == depth as usize {
                    if let Some(&smallest_seq_num_instance) = s.samples.front() {
                        if data_writer.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
                            if let RtpsWriterKind::Stateful(w) = &data_writer.transport_writer {
                                if !w.is_change_acknowledged(smallest_seq_num_instance) {
                                    if data_writer.acknowledgement_notification.is_some() {
                                        reply_sender.send(Err(DdsError::Error(String::from(
                                            "Another writer already waiting for acknowledgements.",
                                        ))));
                                        return;
                                    }
                                    let max_blocking_time =
                                        data_writer.qos.reliability.max_blocking_time;
                                    let (
                                        acknowledgment_notification_sender,
                                        acknowledgment_notification_receiver,
                                    ) = oneshot::<()>();
                                    data_writer.acknowledgement_notification =
                                        Some(acknowledgment_notification_sender);
                                    let participant_handle =
                                        self.domain_participant.instance_handle;
                                    let dcps_sender = self.dcps_sender;
                                    let mut timer_handle = runtime.timer();
                                    let publisher_handle = *publisher_handle;
                                    let data_writer_handle = *data_writer_handle;
                                    let dynamic_data = dynamic_data.clone();
                                    runtime.spawner().spawn(async move {
                                        if let DurationKind::Finite(t) = max_blocking_time {
                                            let max_blocking_time_wait =
                                                timer_handle.delay(t.into());
                                            match select_future(
                                                acknowledgment_notification_receiver,
                                                max_blocking_time_wait,
                                            )
                                            .await
                                            {
                                                Either::A(_) => {
                                                    dcps_sender
                                                        .send(DcpsMail::Writer(
                                                            WriterServiceMail::WriteWTimestamp {
                                                                participant_handle,
                                                                publisher_handle,
                                                                data_writer_handle,
                                                                dynamic_data,
                                                                timestamp,
                                                                reply_sender,
                                                            },
                                                        ))
                                                        .await;
                                                }
                                                Either::B(_) => {
                                                    reply_sender.send(Err(DdsError::Timeout))
                                                }
                                            };
                                        } else {
                                            acknowledgment_notification_receiver.await.ok();
                                            dcps_sender
                                                .send(DcpsMail::Writer(
                                                    WriterServiceMail::WriteWTimestamp {
                                                        participant_handle,
                                                        publisher_handle,
                                                        data_writer_handle,
                                                        dynamic_data,
                                                        timestamp,
                                                        reply_sender,
                                                    },
                                                ))
                                                .await;
                                        }
                                    });
                                    return;
                                }
                            }
                        }
                    }
                    if let Some(smallest_seq_num_instance) = s.samples.pop_front() {
                        data_writer
                            .transport_writer
                            .remove_change(smallest_seq_num_instance);
                    }
                }
            }
        }

        let write_result = data_writer.write_w_timestamp(
            instance_handle,
            serialized_data,
            timestamp,
            now,
            self.transport.message_writer.as_ref(),
            runtime,
        );
        if write_result.is_err() {
            reply_sender.send(write_result);
            return;
        }

        let dcps_sender_clone = self.dcps_sender;
        let participant_handle = self.domain_participant.instance_handle;
        if let DurationKind::Finite(deadline_missed_period) = data_writer.qos.deadline.period {
            let mut timer_handle = runtime.timer();
            let publisher_handle = *publisher_handle;
            let data_writer_handle = *data_writer_handle;
            runtime.spawner().spawn(async move {
                loop {
                    timer_handle.delay(deadline_missed_period.into()).await;
                    dcps_sender_clone
                        .send(DcpsMail::Event(EventServiceMail::OfferedDeadlineMissed {
                            participant_handle,
                            publisher_handle,
                            data_writer_handle,
                            change_instance_handle: instance_handle,
                        }))
                        .await;
                }
            });
        }

        let sequence_number = data_writer.last_change_sequence_number;

        if let DurationKind::Finite(lifespan_duration) = data_writer.qos.lifespan.duration {
            let sleep_duration = timestamp - now + lifespan_duration;
            if sleep_duration <= Duration::new(0, 0) {
                reply_sender.send(Ok(()));
                return;
            }

            let dcps_sender_clone = self.dcps_sender;
            let mut timer_handle = runtime.timer();
            let publisher_handle = *publisher_handle;
            let data_writer_handle = *data_writer_handle;
            runtime.spawner().spawn(async move {
                timer_handle.delay(sleep_duration.into()).await;
                dcps_sender_clone
                    .send(DcpsMail::Message(MessageServiceMail::RemoveWriterChange {
                        participant_handle,
                        publisher_handle,
                        data_writer_handle,
                        sequence_number,
                    }))
                    .await;
            });
        }

        reply_sender.send(Ok(()));
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn dispose_w_timestamp(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        dynamic_data: &DynamicData<'static>,
        timestamp: Time,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_writer.dispose_w_timestamp(
            dynamic_data,
            timestamp,
            self.transport.message_writer.as_ref(),
            runtime,
        )
    }

    #[tracing::instrument(skip(self))]
    pub fn get_offered_deadline_missed_status(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.get_offered_deadline_missed_status())
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn enable_data_writer(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_writer.enabled {
            data_writer.enabled = true;

            let discovered_reader_list: Vec<_> =
                self.domain_participant.discovered_reader_list.to_vec();
            for discovered_reader_data in discovered_reader_list {
                self.add_discovered_reader(
                    &discovered_reader_data,
                    publisher_handle,
                    data_writer_handle,
                );
            }

            self.announce_data_writer(publisher_handle, data_writer_handle, runtime);
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn set_data_writer_qos(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        qos: QosKind<DataWriterQos>,
        runtime: &impl DdsRuntime,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => publisher.default_datawriter_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;
        if data_writer.enabled {
            data_writer.qos.check_immutability(&qos)?;
        }
        data_writer.qos = qos;

        if data_writer.enabled {
            self.announce_data_writer(publisher_handle, data_writer_handle, runtime);
        }
        Ok(())
    }

    // This is a slighlty special function in the send that the answer is sent from
    // here directly because the reply sender is used as the notification mechanism
    // to notify the caller that all the changes are acknowledged.
    #[tracing::instrument(skip(self, notify_sender))]
    pub fn notify_acknowledgments(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        notify_sender: OneshotSender<DdsResult<()>>,
    ) {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return notify_sender.send(Err(DdsError::AlreadyDeleted));
        };

        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return notify_sender.send(Err(DdsError::AlreadyDeleted));
        };

        match &data_writer.transport_writer {
            RtpsWriterKind::Stateful(w) => {
                if w.is_change_acknowledged(data_writer.last_change_sequence_number) {
                    notify_sender.send(Ok(()));
                } else {
                    data_writer
                        .wait_for_acknowledgments_notification
                        .push(notify_sender);
                }
            }
            RtpsWriterKind::Stateless(_) => notify_sender.send(Ok(())),
        }
    }
}
