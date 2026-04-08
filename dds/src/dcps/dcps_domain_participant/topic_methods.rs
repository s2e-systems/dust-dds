use alloc::string::String;

use crate::{
    dcps::{
        dcps_domain_participant::{DcpsDomainParticipant, TopicDescriptionKind},
        status_condition_mail::DcpsStatusConditionMail,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
    runtime::DdsRuntime,
    xtypes::dynamic_type::DynamicType,
};

impl<R: DdsRuntime> DcpsDomainParticipant<R> {
    #[tracing::instrument(skip(self))]
    pub async fn get_inconsistent_topic_status(
        &mut self,
        topic_name: String,
    ) -> DdsResult<InconsistentTopicStatus> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let status = topic.inconsistent_topic_status.clone();
        topic.inconsistent_topic_status.total_count_change = 0;
        topic
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::InconsistentTopic,
            })
            .await;

        Ok(status)
    }

    #[tracing::instrument(skip(self))]
    pub fn set_topic_qos(
        &mut self,
        topic_name: String,
        topic_qos: QosKind<TopicQos>,
    ) -> DdsResult<()> {
        let qos = match topic_qos {
            QosKind::Default => self.domain_participant.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;

        if topic.enabled
            && (topic.qos.durability != qos.durability
                || topic.qos.liveliness != qos.liveliness
                || topic.qos.reliability != qos.reliability
                || topic.qos.destination_order != qos.destination_order
                || topic.qos.history != qos.history
                || topic.qos.resource_limits != qos.resource_limits
                || topic.qos.ownership != qos.ownership)
        {
            return Err(DdsError::ImmutablePolicy);
        }

        topic.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_topic_qos(&mut self, topic_name: String) -> DdsResult<TopicQos> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(topic.qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable_topic(&mut self, topic_name: String) -> DdsResult<()> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !topic.enabled {
            topic.enabled = true;
            self.announce_topic(topic_name).await;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_type_support(&mut self, topic_name: String) -> DdsResult<&'static dyn DynamicType> {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter_mut()
            .find(|x| x.topic_name() == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(topic.type_support)
    }
}
