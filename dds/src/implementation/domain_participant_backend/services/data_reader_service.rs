use core::{future::Future, pin::Pin};

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    implementation::{
        any_data_reader_listener::AnyDataReaderListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            services::message_service::{self, IsHistoricalDataReceived},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        status::{StatusKind, SubscriptionMatchedStatus},
        time::Duration,
    },
    rtps::messages::submessage_elements::Data,
    runtime::actor::{ActorAddress, Mail, MailHandler},
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
};

pub struct Read {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_samples: i32,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub specific_instance_handle: Option<InstanceHandle>,
}
impl Mail for Read {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<Read> for DomainParticipantActor {
    fn handle(&mut self, message: Read) -> <Read as Mail>::Result {
        let subscriber =
            if message.subscriber_handle == self.domain_participant.instance_handle() {
                Some(self.domain_participant.builtin_subscriber_mut())
            } else {
                self.domain_participant
                    .get_mut_subscriber(message.subscriber_handle)
            }
            .ok_or(DdsError::AlreadyDeleted)?;

        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        data_reader.read(
            message.max_samples,
            &message.sample_states,
            &message.view_states,
            &message.instance_states,
            message.specific_instance_handle,
        )
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
}
impl Mail for Take {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<Take> for DomainParticipantActor {
    fn handle(&mut self, message: Take) -> <Take as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        data_reader.take(
            message.max_samples,
            message.sample_states,
            message.view_states,
            message.instance_states,
            message.specific_instance_handle,
        )
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
}
impl Mail for ReadNextInstance {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<ReadNextInstance> for DomainParticipantActor {
    fn handle(&mut self, message: ReadNextInstance) -> <ReadNextInstance as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        data_reader.read_next_instance(
            message.max_samples,
            message.previous_handle,
            &message.sample_states,
            &message.view_states,
            &message.instance_states,
        )
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
}
impl Mail for TakeNextInstance {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<TakeNextInstance> for DomainParticipantActor {
    fn handle(&mut self, message: TakeNextInstance) -> <TakeNextInstance as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        data_reader.take_next_instance(
            message.max_samples,
            message.previous_handle,
            message.sample_states,
            message.view_states,
            message.instance_states,
        )
    }
}

pub struct GetSubscriptionMatchedStatus {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for GetSubscriptionMatchedStatus {
    type Result = DdsResult<SubscriptionMatchedStatus>;
}
impl MailHandler<GetSubscriptionMatchedStatus> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetSubscriptionMatchedStatus,
    ) -> <GetSubscriptionMatchedStatus as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        Ok(data_reader.get_subscription_matched_status())
    }
}

pub struct WaitForHistoricalData {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub max_wait: Duration,
}
impl Mail for WaitForHistoricalData {
    type Result = Pin<Box<dyn Future<Output = DdsResult<()>> + Send>>;
}
impl MailHandler<WaitForHistoricalData> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: WaitForHistoricalData,
    ) -> <WaitForHistoricalData as Mail>::Result {
        let timer_handle = self.timer_driver.handle();

        Box::pin(async move {
            timer_handle
                .timeout(
                    message.max_wait.into(),
                    Box::pin(async move {
                        loop {
                            if message
                                .participant_address
                                .send_actor_mail(IsHistoricalDataReceived {
                                    subscriber_handle: message.subscriber_handle,
                                    data_reader_handle: message.data_reader_handle,
                                })?
                                .receive_reply()
                                .await?
                            {
                                return Ok(());
                            }
                        }
                    }),
                )
                .await
                .map_err(|_| DdsError::Timeout)?
        })
    }
}

pub struct GetMatchedPublicationData {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub publication_handle: InstanceHandle,
}
impl Mail for GetMatchedPublicationData {
    type Result = DdsResult<PublicationBuiltinTopicData>;
}
impl MailHandler<GetMatchedPublicationData> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetMatchedPublicationData,
    ) -> <GetMatchedPublicationData as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !data_reader.enabled() {
            return Err(DdsError::NotEnabled);
        }

        data_reader
            .get_matched_publication_data(&message.publication_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }
}

pub struct GetMatchedPublications {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for GetMatchedPublications {
    type Result = DdsResult<Vec<InstanceHandle>>;
}
impl MailHandler<GetMatchedPublications> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetMatchedPublications,
    ) -> <GetMatchedPublications as Mail>::Result {
        Ok(self
            .domain_participant
            .get_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_matched_publications())
    }
}

pub struct SetDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub qos: QosKind<DataReaderQos>,
}
impl Mail for SetDataReaderQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDataReaderQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDataReaderQos) -> <SetDataReaderQos as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => subscriber.default_data_reader_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let subscriber_qos = subscriber.qos().clone();
        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        data_reader.set_qos(qos)?;

        if data_reader.enabled() {
            // let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
            //     key: BuiltInTopicKey {
            //         value: data_reader.transport_reader().guid(),
            //     },
            //     participant_key: BuiltInTopicKey { value: [0; 16] },
            //     topic_name: data_reader.topic_name().to_owned(),
            //     type_name: data_reader.type_name().to_owned(),
            //     durability: data_reader.qos().durability.clone(),
            //     deadline: data_reader.qos().deadline.clone(),
            //     latency_budget: data_reader.qos().latency_budget.clone(),
            //     liveliness: data_reader.qos().liveliness.clone(),
            //     reliability: data_reader.qos().reliability.clone(),
            //     ownership: data_reader.qos().ownership.clone(),
            //     destination_order: data_reader.qos().destination_order.clone(),
            //     user_data: data_reader.qos().user_data.clone(),
            //     time_based_filter: data_reader.qos().time_based_filter.clone(),
            //     presentation: subscriber_qos.presentation,
            //     partition: subscriber_qos.partition,
            //     topic_data: self.domain_participant.topic_list[data_reader.topic_name()]
            //         .qos()
            //         .topic_data
            //         .clone(),
            //     group_data: subscriber_qos.group_data,
            //     representation: data_reader.qos().representation.clone(),
            // };

            // self.domain_participant
            //     .announce_created_or_modified_datareader(subscription_builtin_topic_data)?;
            todo!()
        }

        Ok(())
    }
}

pub struct GetDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for GetDataReaderQos {
    type Result = DdsResult<DataReaderQos>;
}
impl MailHandler<GetDataReaderQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDataReaderQos) -> <GetDataReaderQos as Mail>::Result {
        todo!()
    }
}

pub struct Enable {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for Enable {
    type Result = DdsResult<()>;
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) -> <Enable as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !data_reader.enabled() {
            data_reader.enable();
            message
                .participant_address
                .send_actor_mail(message_service::AnnounceDataReader {
                    subscriber_handle: message.subscriber_handle,
                    data_reader_handle: message.data_reader_handle,
                })
                .ok();
        }
        Ok(())
    }
}

pub struct GetDataReaderInstanceHandle {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for GetDataReaderInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetDataReaderInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDataReaderInstanceHandle,
    ) -> <GetDataReaderInstanceHandle as Mail>::Result {
        todo!()
    }
}

pub struct SetDataReaderListener {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetDataReaderListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDataReaderListener> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDataReaderListener,
    ) -> <SetDataReaderListener as Mail>::Result {
        todo!()
    }
}
