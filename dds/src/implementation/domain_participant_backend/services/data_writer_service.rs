use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        domain_participant_backend::domain_participant_actor::DomainParticipantActor,
        listeners::data_writer_listener::DataWriterListenerActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        status::StatusKind,
    },
    runtime::{
        actor::{Actor, MailHandler},
        oneshot::OneshotSender,
    },
};

pub struct GetMatchedSubscriptionData {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub subscription_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<SubscriptionBuiltinTopicData>>,
}
impl MailHandler<GetMatchedSubscriptionData> for DomainParticipantActor {
    fn handle(&mut self, message: GetMatchedSubscriptionData) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(
            data_writer
                .get_matched_subscription_data(&message.subscription_handle)
                .ok_or(DdsError::BadParameter)
                .cloned(),
        );
    }
}

pub struct GetMatchedSubscriptions {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<Vec<InstanceHandle>>>,
}
impl MailHandler<GetMatchedSubscriptions> for DomainParticipantActor {
    fn handle(&mut self, message: GetMatchedSubscriptions) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message
            .reply_sender
            .send(Ok(data_writer.get_matched_subscriptions()))
    }
}

pub struct GetDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<DataWriterQos>>,
}
impl MailHandler<GetDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDataWriterQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == message.data_writer_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(data_writer.qos().clone()));
    }
}

pub struct SetListener {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub listener_mask: Vec<StatusKind>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) {
        let listener = message.listener.map(|l| {
            Actor::spawn(
                DataWriterListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle) else {
            return;
        };

        data_writer.set_listener(listener, message.listener_mask);

        message.reply_sender.send(Ok(()));
    }
}
