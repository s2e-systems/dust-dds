use std::sync::Arc;

use crate::{
    implementation::domain_participant_backend::domain_participant_actor::DomainParticipantActor,
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{QosKind, TopicQos},
        status::InconsistentTopicStatus,
    },
    runtime::{
        actor::{ActorAddress, MailHandler},
        oneshot::OneshotSender,
    },
    xtypes::dynamic_type::DynamicType,
};

use super::discovery_service;

pub struct GetInconsistentTopicStatus {
    pub topic_name: String,
    pub reply_sender: OneshotSender<DdsResult<InconsistentTopicStatus>>,
}
impl MailHandler<GetInconsistentTopicStatus> for DomainParticipantActor {
    fn handle(&mut self, message: GetInconsistentTopicStatus) {
        let Some(topic) = self.domain_participant.get_mut_topic(&message.topic_name) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message
            .reply_sender
            .send(Ok(topic.get_inconsistent_topic_status()))
    }
}

pub struct SetQos {
    pub topic_name: String,
    pub topic_qos: QosKind<TopicQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) {
        let qos = match message.topic_qos {
            QosKind::Default => self.domain_participant.get_default_topic_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(topic) = self.domain_participant.get_mut_topic(&message.topic_name) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        topic.set_qos(qos);

        message.reply_sender.send(Ok(()));
    }
}

pub struct GetQos {
    pub topic_name: String,
    pub reply_sender: OneshotSender<DdsResult<TopicQos>>,
}
impl MailHandler<GetQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetQos) {
        let Some(topic) = self.domain_participant.get_mut_topic(&message.topic_name) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(topic.qos().clone()));
    }
}

pub struct Enable {
    pub topic_name: String,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) {
        let Some(topic) = self.domain_participant.get_mut_topic(&message.topic_name) else {
            // message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        if !topic.enabled() {
            topic.enable();
            message
                .participant_address
                .send_actor_mail(discovery_service::AnnounceTopic {
                    topic_name: message.topic_name,
                })
                .ok();
        }

        // message.reply_sender.send(Ok(()));
    }
}

pub struct GetTypeSupport {
    pub topic_name: String,
    pub reply_sender: OneshotSender<DdsResult<Arc<dyn DynamicType + Send + Sync>>>,
}
impl MailHandler<GetTypeSupport> for DomainParticipantActor {
    fn handle(&mut self, message: GetTypeSupport) {
        let Some(topic) = self.domain_participant.get_mut_topic(&message.topic_name) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message.reply_sender.send(Ok(topic.type_support().clone()));
    }
}
