use alloc::{boxed::Box, string::String, vec::Vec};
use core::pin::Pin;

use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps::{
        channels::oneshot::oneshot,
        dcps_domain_participant::{DcpsDomainParticipant, RtpsReaderKind, poll_timeout},
        dcps_mail::{DcpsMail, MessageServiceMail},
        listeners::data_reader_listener::DcpsDataReaderListener,
        status_condition_mail::DcpsStatusConditionMail,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        qos_policy::DurabilityQosPolicyKind,
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
        status::{StatusKind, SubscriptionMatchedStatus},
        time::Duration,
    },
    runtime::DdsRuntime,
    xtypes::dynamic_type::DynamicData,
};

impl<R: DdsRuntime> DcpsDomainParticipant<R> {
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn read(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<DynamicData>, SampleInfo)>> {
        let subscriber = if subscriber_handle == self.domain_participant.instance_handle {
            Some(&mut self.domain_participant.builtin_subscriber)
        } else {
            self.domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
        };

        let Some(subscriber) = subscriber else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_reader
            .read(
                max_samples,
                &sample_states,
                &view_states,
                &instance_states,
                specific_instance_handle,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn take(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<DynamicData>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .take(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn read_next_instance(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<DynamicData>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .read_next_instance(
                max_samples,
                previous_handle,
                &sample_states,
                &view_states,
                &instance_states,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn take_next_instance(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<DynamicData>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .take_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_subscription_matched_status(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionMatchedStatus> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let status = data_reader.get_subscription_matched_status();
        data_reader
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::SubscriptionMatched,
            })
            .await;
        Ok(status)
    }

    //#[tracing::instrument(skip(self, dcps_sender))]
    pub fn wait_for_historical_data(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_wait: Duration,
    ) -> Pin<Box<dyn Future<Output = DdsResult<()>> + Send>> {
        let participant_handle = self.domain_participant.instance_handle;
        let timer_handle = self.timer_handle.clone();
        let dcps_sender = self.dcps_sender.clone();
        Box::pin(async move {
            poll_timeout(
                timer_handle,
                max_wait.into(),
                Box::pin(async move {
                    loop {
                        let (reply_sender, reply_receiver) = oneshot();
                        dcps_sender
                            .send(DcpsMail::Message(
                                MessageServiceMail::IsHistoricalDataReceived {
                                    participant_handle,
                                    subscriber_handle,
                                    data_reader_handle,
                                    reply_sender,
                                },
                            ))
                            .await?;

                        let reply = reply_receiver.await;
                        match reply {
                            Ok(historical_data_received) => match historical_data_received {
                                Ok(true) => return Ok(()),
                                Ok(false) => (),
                                Err(e) => return Err(e),
                            },
                            Err(_) => return Err(DdsError::Error(String::from("Channel error"))),
                        }
                    }
                }),
            )
            .await?
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_publication_data(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            return Err(DdsError::NotEnabled);
        }

        data_reader
            .matched_publication_list
            .iter()
            .find(|x| &x.key().value == publication_handle.as_ref())
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_publications(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_reader.get_matched_publications())
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;
        if data_reader.enabled {
            data_reader.qos.check_immutability(&qos)?
        }

        data_reader.qos = qos;

        if data_reader.enabled {
            self.announce_data_reader(subscriber_handle, data_reader_handle)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<DataReaderQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_reader.qos.clone())
    }

    #[tracing::instrument(skip(self, dcps_listener))]
    pub fn set_data_reader_listener(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        dcps_listener: Option<DcpsDataReaderListener>,
        listener_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&self.spawner_handle));
        data_reader.listener_sender = listener_sender;
        data_reader.listener_mask = listener_mask;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn is_historical_data_received(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            return Err(DdsError::NotEnabled);
        };

        match data_reader.qos.durability.kind {
            DurabilityQosPolicyKind::Volatile => {
                return Err(DdsError::IllegalOperation);
            }
            DurabilityQosPolicyKind::TransientLocal
            | DurabilityQosPolicyKind::Transient
            | DurabilityQosPolicyKind::Persistent => (),
        };

        if let RtpsReaderKind::Stateful(r) = &data_reader.transport_reader {
            Ok(r.is_historical_data_received())
        } else {
            Ok(true)
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            data_reader.enabled = true;

            let discovered_writer_list: Vec<_> =
                self.domain_participant.discovered_writer_list.to_vec();
            for discovered_writer_data in discovered_writer_list {
                self.add_discovered_writer(
                    discovered_writer_data,
                    subscriber_handle,
                    data_reader_handle,
                )
                .await;
            }

            self.announce_data_reader(subscriber_handle, data_reader_handle)
                .await;
        }
        Ok(())
    }
}
