use alloc::{collections::VecDeque, string::String, vec::Vec};

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps::{
        channels::{
            mpsc::MpscSender,
            oneshot::{OneshotSender, oneshot},
        },
        dcps_domain_participant::{
            DcpsDomainParticipant, InstancePublicationTime, InstanceSamples, TransportWriterKind,
            serialize,
        },
        dcps_mail::{DcpsMail, EventServiceMail, MessageServiceMail, WriterServiceMail},
        listeners::data_writer_listener::DcpsDataWriterListener,
        status_condition_mail::DcpsStatusConditionMail,
        xtypes_glue::key_and_instance_handle::get_instance_handle_from_dynamic_data,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        qos_policy::{HistoryQosPolicyKind, Length, ReliabilityQosPolicyKind},
        status::{OfferedDeadlineMissedStatus, PublicationMatchedStatus, StatusKind},
        time::{Duration, DurationKind, Time},
    },
    runtime::{DdsRuntime, Either, Spawner, Timer, select_future},
    transport::types::{CacheChange, ChangeKind},
    xtypes::dynamic_type::DynamicData,
};

impl<R: DdsRuntime> DcpsDomainParticipant<R> {
    #[tracing::instrument(skip(self))]
    pub async fn get_publication_matched_status(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<PublicationMatchedStatus> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let status = data_writer.get_publication_matched_status();

        data_writer
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            })
            .await;
        Ok(status)
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn set_listener_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataWriterListener>,
        listener_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Ok(());
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Ok(());
        };

        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        data_writer.listener_sender = listener_sender;
        data_writer.listener_mask = listener_mask;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_data_writer_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_subscriptions(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
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
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
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

    #[tracing::instrument(skip(self))]
    pub async fn unregister_instance(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_writer
            .unregister_w_timestamp(
                dynamic_data,
                timestamp,
                self.transport.message_writer.as_ref(),
                &self.clock_handle,
            )
            .await
    }

    #[tracing::instrument(skip(self))]
    pub fn lookup_instance(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
    ) -> DdsResult<Option<InstanceHandle>> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !data_writer.enabled {
            return Err(DdsError::NotEnabled);
        }

        let instance_handle = match get_instance_handle_from_dynamic_data(dynamic_data) {
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

    #[tracing::instrument(skip(self, dcps_sender, reply_sender))]
    pub async fn write_w_timestamp(
        &mut self,
        dcps_sender: MpscSender<DcpsMail>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
        reply_sender: OneshotSender<DdsResult<()>>,
    ) {
        let now = self.get_current_time();
        let Some(publisher) = core::iter::once(&mut self.domain_participant.builtin_publisher)
            .chain(&mut self.domain_participant.user_defined_publisher_list)
            .find(|x| x.instance_handle == publisher_handle)
        else {
            reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        if !data_writer.enabled {
            reply_sender.send(Err(DdsError::NotEnabled));
            return;
        }

        let instance_handle = match get_instance_handle_from_dynamic_data(dynamic_data.clone()) {
            Ok(h) => h,
            Err(e) => {
                reply_sender.send(Err(e.into()));
                return;
            }
        };

        if !data_writer
            .registered_instance_list
            .contains(&instance_handle)
        {
            if data_writer.registered_instance_list.len()
                < data_writer.qos.resource_limits.max_instances
            {
                data_writer.registered_instance_list.push(instance_handle);
            } else {
                reply_sender.send(Err(DdsError::OutOfResources));
                return;
            }
        }

        if let Length::Limited(max_instances) = data_writer.qos.resource_limits.max_instances {
            if !data_writer
                .instance_samples
                .iter()
                .any(|x| x.instance == instance_handle)
                && data_writer.instance_samples.len() == max_instances as usize
            {
                reply_sender.send(Err(DdsError::OutOfResources));
                return;
            }
        }

        if let Length::Limited(max_samples_per_instance) =
            data_writer.qos.resource_limits.max_samples_per_instance
        {
            // If the history Qos guarantess that the number of samples
            // is below the limit there is no need to check
            match data_writer.qos.history.kind {
                HistoryQosPolicyKind::KeepLast(depth)
                    if depth as i32 <= max_samples_per_instance => {}
                _ => {
                    if let Some(s) = data_writer
                        .instance_samples
                        .iter()
                        .find(|x| x.instance == instance_handle)
                    {
                        // Only Alive changes count towards the resource limits
                        if s.samples.len() >= max_samples_per_instance as usize {
                            reply_sender.send(Err(DdsError::OutOfResources));
                            return;
                        }
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = data_writer.qos.resource_limits.max_samples {
            let total_samples = data_writer
                .instance_samples
                .iter()
                .fold(0, |acc, x| acc + x.samples.len());

            if total_samples >= max_samples as usize {
                reply_sender.send(Err(DdsError::OutOfResources));
                return;
            }
        }

        let serialized_data = match serialize(&dynamic_data, &data_writer.qos.representation) {
            Ok(s) => s,
            Err(e) => {
                reply_sender.send(Err(e));
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
                            if let TransportWriterKind::Stateful(w) = &data_writer.transport_writer
                            {
                                if !w.is_change_acknowledged(smallest_seq_num_instance) {
                                    if data_writer.acknowledgement_notification.is_some() {
                                        reply_sender.send(Err(DdsError::Error(String::from(
                                            "Another writer already waiting for acknowledgements.",
                                        ))));
                                        return;
                                    }
                                    let mut timer_handle = self.timer_handle.clone();
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
                                    self.spawner_handle.spawn(async move {
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
                                                                dcps_sender: dcps_sender.clone(),
                                                                reply_sender,
                                                            },
                                                        ))
                                                        .await
                                                        .ok();
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
                                                        dcps_sender: dcps_sender.clone(),
                                                        reply_sender,
                                                    },
                                                ))
                                                .await
                                                .ok();
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
                            .remove_change(smallest_seq_num_instance)
                            .await;
                    }
                }
            }
        }

        data_writer.last_change_sequence_number += 1;
        let change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: data_writer.transport_writer.guid(),
            sequence_number: data_writer.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_data.into(),
        };
        let seq_num = change.sequence_number;

        if seq_num > data_writer.max_seq_num.unwrap_or(0) {
            data_writer.max_seq_num = Some(seq_num)
        }

        match data_writer
            .instance_publication_time
            .iter_mut()
            .find(|x| x.instance == instance_handle)
        {
            Some(x) => {
                if x.last_write_time < timestamp {
                    x.last_write_time = timestamp;
                }
            }
            None => data_writer
                .instance_publication_time
                .push(InstancePublicationTime {
                    instance: instance_handle,
                    last_write_time: timestamp,
                }),
        }

        match data_writer
            .instance_samples
            .iter_mut()
            .find(|x| x.instance == instance_handle)
        {
            Some(s) => s.samples.push_back(change.sequence_number),
            None => {
                let s = InstanceSamples {
                    instance: instance_handle,
                    samples: VecDeque::from([change.sequence_number]),
                };
                data_writer.instance_samples.push(s);
            }
        }

        let dcps_sender_clone = dcps_sender.clone();
        let participant_handle = self.domain_participant.instance_handle;
        if let DurationKind::Finite(deadline_missed_period) = data_writer.qos.deadline.period {
            let mut timer_handle = self.timer_handle.clone();
            self.spawner_handle.spawn(async move {
                loop {
                    timer_handle.delay(deadline_missed_period.into()).await;
                    dcps_sender_clone
                        .send(DcpsMail::Event(EventServiceMail::OfferedDeadlineMissed {
                            participant_handle,
                            publisher_handle,
                            data_writer_handle,
                            change_instance_handle: instance_handle,
                            dcps_sender: dcps_sender_clone.clone(),
                        }))
                        .await
                        .ok();
                }
            });
        }

        let sequence_number = data_writer.last_change_sequence_number;

        if let DurationKind::Finite(lifespan_duration) = data_writer.qos.lifespan.duration {
            let sleep_duration = timestamp - now + lifespan_duration;
            let mut timer_handle = self.timer_handle.clone();
            if sleep_duration <= Duration::new(0, 0) {
                reply_sender.send(Ok(()));
                return;
            }

            let dcps_sender_clone = dcps_sender.clone();
            self.spawner_handle.spawn(async move {
                timer_handle.delay(sleep_duration.into()).await;
                dcps_sender_clone
                    .send(DcpsMail::Message(MessageServiceMail::RemoveWriterChange {
                        participant_handle,
                        publisher_handle,
                        data_writer_handle,
                        sequence_number,
                    }))
                    .await
                    .ok();
            });
        }

        data_writer
            .transport_writer
            .add_change(
                change,
                self.transport.message_writer.as_ref(),
                &self.clock_handle,
            )
            .await;

        reply_sender.send(Ok(()));
    }

    #[tracing::instrument(skip(self))]
    pub async fn dispose_w_timestamp(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dynamic_data: DynamicData,
        timestamp: Time,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_writer
            .dispose_w_timestamp(
                dynamic_data,
                timestamp,
                self.transport.message_writer.as_ref(),
                &self.clock_handle,
            )
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_offered_deadline_missed_status(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.get_offered_deadline_missed_status().await)
    }

    #[tracing::instrument(skip(self, dcps_sender))]
    pub async fn enable_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        dcps_sender: MpscSender<DcpsMail>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_writer.enabled {
            data_writer.enabled = true;

            let discovered_reader_list: Vec<_> =
                self.domain_participant.discovered_reader_list.to_vec();
            for discovered_reader_data in discovered_reader_list {
                self.add_discovered_reader(
                    discovered_reader_data,
                    publisher_handle,
                    data_writer_handle,
                    dcps_sender.clone(),
                )
                .await;
            }

            self.announce_data_writer(publisher_handle, data_writer_handle, dcps_sender)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_data_writer_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
        dcps_sender: MpscSender<DcpsMail>,
    ) -> DdsResult<()> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
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
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;
        if data_writer.enabled {
            data_writer.qos.check_immutability(&qos)?;
        }
        data_writer.qos = qos;

        if data_writer.enabled {
            self.announce_data_writer(publisher_handle, data_writer_handle, dcps_sender)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn are_all_changes_acknowledged(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_writer) = publisher
            .data_writer_list
            .iter()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_writer.are_all_changes_acknowledged().await)
    }
}
