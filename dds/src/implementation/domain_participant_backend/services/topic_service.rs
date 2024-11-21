use std::sync::Arc;

use crate::{
    implementation::domain_participant_backend::domain_participant_actor::DomainParticipantActor,
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{QosKind, TopicQos},
        status::InconsistentTopicStatus,
    },
    runtime::actor::{ActorAddress, Mail, MailHandler},
    xtypes::dynamic_type::DynamicType,
};

use super::discovery_service;

pub struct GetInconsistentTopicStatus {
    pub topic_name: String,
}
impl Mail for GetInconsistentTopicStatus {
    type Result = DdsResult<InconsistentTopicStatus>;
}
impl MailHandler<GetInconsistentTopicStatus> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetInconsistentTopicStatus,
    ) -> <GetInconsistentTopicStatus as Mail>::Result {
        Ok(self
            .domain_participant
            .get_mut_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_inconsistent_topic_status())
    }
}

pub struct SetQos {
    pub topic_name: String,
    pub topic_qos: QosKind<TopicQos>,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        let qos = match message.topic_qos {
            QosKind::Default => self.domain_participant.get_default_topic_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let topic = self
            .domain_participant
            .get_mut_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        topic.set_qos(qos)
    }
}

pub struct GetQos {
    pub topic_name: String,
}
impl Mail for GetQos {
    type Result = DdsResult<TopicQos>;
}
impl MailHandler<GetQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetQos) -> <GetQos as Mail>::Result {
        Ok(self
            .domain_participant
            .get_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .qos()
            .clone())
    }
}

pub struct Enable {
    pub topic_name: String,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for Enable {
    type Result = DdsResult<()>;
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) -> <Enable as Mail>::Result {
        let topic = self
            .domain_participant
            .get_mut_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !topic.enabled() {
            topic.enable();
            message
                .participant_address
                .send_actor_mail(discovery_service::AnnounceTopic {
                    topic_name: message.topic_name,
                })
                .ok();
        }

        Ok(())
    }
}

pub struct GetTypeSupport {
    pub topic_name: String,
}
impl Mail for GetTypeSupport {
    type Result = DdsResult<Arc<dyn DynamicType + Send + Sync>>;
}
impl MailHandler<GetTypeSupport> for DomainParticipantActor {
    fn handle(&mut self, message: GetTypeSupport) -> <GetTypeSupport as Mail>::Result {
        Ok(self
            .domain_participant
            .get_mut_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .type_support()
            .clone())
    }
}
