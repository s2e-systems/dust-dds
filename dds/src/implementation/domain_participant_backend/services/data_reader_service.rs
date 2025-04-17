use core::{future::Future, pin::Pin};
use std::sync::Arc;

use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        any_data_reader_listener::AnyDataReaderListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            services::message_service::IsHistoricalDataReceived,
        },
        listeners::data_reader_listener::DataReaderListenerActor,
        status_condition::status_condition_actor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        status::{StatusKind, SubscriptionMatchedStatus},
        time::Duration,
    },
    runtime::{
        actor::{Actor, ActorAddress, MailHandler},
        oneshot::{oneshot, OneshotSender},
    },
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
};

use super::discovery_service;

pub struct Read {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub specific_instance_handle: Option<InstanceHandle>,
    pub reply_sender: OneshotSender<DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>>>,
}
impl MailHandler<Read> for DomainParticipantActor {
    fn handle(&mut self, message: Read) {
        let subscriber = if message.subscriber_handle == self.domain_participant.instance_handle() {
            Some(self.domain_participant.builtin_subscriber_mut())
        } else {
            self.domain_participant
                .get_mut_subscriber(message.subscriber_handle)
        };

        let Some(subscriber) = subscriber else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(data_reader.read(
            message.max_samples,
            &message.sample_states,
            &message.view_states,
            &message.instance_states,
            message.specific_instance_handle,
        ));
    }
}

pub struct Take {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub specific_instance_handle: Option<InstanceHandle>,
    pub reply_sender: OneshotSender<DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>>>,
}
impl MailHandler<Take> for DomainParticipantActor {
    fn handle(&mut self, message: Take) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message.reply_sender.send(data_reader.take(
            message.max_samples,
            message.sample_states,
            message.view_states,
            message.instance_states,
            message.specific_instance_handle,
        ));
    }
}

pub struct ReadNextInstance {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub previous_handle: Option<InstanceHandle>,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub reply_sender: OneshotSender<DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>>>,
}
impl MailHandler<ReadNextInstance> for DomainParticipantActor {
    fn handle(&mut self, message: ReadNextInstance) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message.reply_sender.send(data_reader.read_next_instance(
            message.max_samples,
            message.previous_handle,
            &message.sample_states,
            &message.view_states,
            &message.instance_states,
        ));
    }
}

pub struct TakeNextInstance {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub previous_handle: Option<InstanceHandle>,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub reply_sender: OneshotSender<DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>>>,
}
impl MailHandler<TakeNextInstance> for DomainParticipantActor {
    fn handle(&mut self, message: TakeNextInstance) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message.reply_sender.send(data_reader.take_next_instance(
            message.max_samples,
            message.previous_handle,
            message.sample_states,
            message.view_states,
            message.instance_states,
        ));
    }
}

pub struct GetSubscriptionMatchedStatus {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<SubscriptionMatchedStatus>>,
}

impl MailHandler<GetSubscriptionMatchedStatus> for DomainParticipantActor {
    fn handle(&mut self, message: GetSubscriptionMatchedStatus) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let status = data_reader.get_subscription_matched_status();
        data_reader.status_condition().send_actor_mail(
            status_condition_actor::RemoveCommunicationState {
                state: StatusKind::SubscriptionMatched,
            },
        );
        message.reply_sender.send(Ok(status));
    }
}

pub struct WaitForHistoricalData {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_wait: Duration,
    pub reply_sender: OneshotSender<Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>>,
}
impl MailHandler<WaitForHistoricalData> for DomainParticipantActor {
    fn handle(&mut self, message: WaitForHistoricalData) {
        let timer_handle = self.timer_driver.handle();

        message.reply_sender.send(Box::pin(async move {
            timer_handle
                .timeout(
                    message.max_wait.into(),
                    Box::pin(async move {
                        loop {
                            let (reply_sender, reply_receiver) = oneshot();
                            message
                                .participant_address
                                .send_actor_mail(IsHistoricalDataReceived {
                                    subscriber_handle: message.subscriber_handle,
                                    data_reader_handle: message.data_reader_handle,
                                    reply_sender,
                                });

                            let reply = reply_receiver.await;
                            match reply {
                                Ok(historical_data_received) => match historical_data_received {
                                    Ok(true) => return Ok(()),
                                    Ok(false) => (),
                                    Err(e) => return Err(e),
                                },
                                Err(e) => {
                                    return Err(DdsError::Error(format!("Channel error: {:?}", e)))
                                }
                            }
                        }
                    }),
                )
                .await
                .map_err(|_| DdsError::Timeout)?
        }));
    }
}

pub struct GetMatchedPublicationData {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub publication_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<PublicationBuiltinTopicData>>,
}
impl MailHandler<GetMatchedPublicationData> for DomainParticipantActor {
    fn handle(&mut self, message: GetMatchedPublicationData) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.get_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        if !data_reader.enabled() {
            message.reply_sender.send(Err(DdsError::NotEnabled));
            return;
        }

        message.reply_sender.send(
            data_reader
                .get_matched_publication_data(&message.publication_handle)
                .cloned()
                .ok_or(DdsError::BadParameter),
        );
    }
}

pub struct GetMatchedPublications {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
}
impl MailHandler<GetMatchedPublications> for DomainParticipantActor {
    fn handle(&mut self, message: GetMatchedPublications) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.get_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message
            .reply_sender
            .send(Ok(data_reader.get_matched_publications()));
    }
}

pub struct SetQos {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub qos: QosKind<DataReaderQos>,
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}

impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let qos = match message.qos {
            QosKind::Default => subscriber.default_data_reader_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        match data_reader.set_qos(qos) {
            Ok(_) => (),
            Err(e) => {
                message.reply_sender.send(Err(e));
                return;
            }
        };

        if data_reader.enabled() {
            message
                .participant_address
                .send_actor_mail(discovery_service::AnnounceDataReader {
                    subscriber_handle: message.subscriber_handle,
                    data_reader_handle: message.data_reader_handle,
                })
                .ok();
        }

        message.reply_sender.send(Ok(()));
    }
}

pub struct GetQos {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<DataReaderQos>>,
}
impl MailHandler<GetQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetQos) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message.reply_sender.send(Ok(data_reader.qos().clone()))
    }
}

pub struct Enable {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            return;
        };
        if !data_reader.enabled() {
            data_reader.enable();

            for discovered_writer_data in self
                .domain_participant
                .publication_builtin_topic_data_list()
                .cloned()
            {
                message
                    .participant_address
                    .send_actor_mail(discovery_service::AddDiscoveredWriter {
                        discovered_writer_data,
                        subscriber_handle: message.subscriber_handle,
                        data_reader_handle: message.data_reader_handle,
                        participant_address: message.participant_address.clone(),
                    })
                    .ok();
            }

            message
                .participant_address
                .send_actor_mail(discovery_service::AnnounceDataReader {
                    subscriber_handle: message.subscriber_handle,
                    data_reader_handle: message.data_reader_handle,
                })
                .ok();
        }
    }
}

pub struct SetListener {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub listener: Option<Box<dyn AnyDataReaderListener>>,
    pub listener_mask: Vec<StatusKind>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) {
        let listener = message.listener.map(|l| {
            Actor::spawn(
                DataReaderListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        data_reader.set_listener(listener, message.listener_mask);
        message.reply_sender.send(Ok(()));
    }
}
