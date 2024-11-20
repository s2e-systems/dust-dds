use std::sync::Arc;

use crate::{
    implementation::domain_participant_backend::domain_participant_actor::DomainParticipantActor,
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{QosKind, TopicQos},
        status::InconsistentTopicStatus,
    },
    runtime::actor::{Mail, MailHandler},
    xtypes::dynamic_type::DynamicType,
};

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
}
impl Mail for Enable {
    type Result = DdsResult<()>;
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) -> <Enable as Mail>::Result {
        self.domain_participant
            .get_mut_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .enable();
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
