use std::sync::Arc;

use crate::{
    implementation::actor::{Mail, MailHandler},
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{QosKind, TopicQos},
        status::InconsistentTopicStatus,
    },
    xtypes::dynamic_type::DynamicType,
};

use super::domain_participant_actor::DomainParticipantActor;

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
            .topic_list
            .get_mut(&message.topic_name)
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
        let topic = self
            .topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let qos = match message.topic_qos {
            QosKind::Default => self.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };

        topic.set_qos(qos);

        Ok(())
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
            .topic_list
            .get(&message.topic_name)
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
        self.topic_list
            .get_mut(&message.topic_name)
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
            .topic_list
            .get_mut(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?
            .type_support()
            .clone())
    }
}
