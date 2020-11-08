use rust_dds_interface::types::{ReturnCode, InstanceHandle, ReturnCodes};

use crate::infrastructure::status::{InconsistentTopicStatus, StatusMask};
use crate::infrastructure::entity::Entity;
use crate::infrastructure::entity::DomainEntity;
use crate::topic::topic_listener::TopicListener;
use crate::topic::topic_description::TopicDescription;
use crate::domain::DomainParticipant;
use crate::types::DDSType;

use rust_dds_interface::qos::TopicQos;

struct TopicImpl<'topic, T: DDSType>{
    parent_participant: &'topic DomainParticipant,
    topic_name: String,
    topic_data: std::marker::PhantomData<T>,
}

/// Topic is the most basic description of the data to be published and subscribed.
/// A Topic is identified by its name, which must be unique in the whole Domain. In addition (by virtue of extending
/// TopicDescription) it fully specifies the type of the data that can be communicated when publishing or subscribing to the Topic.
/// Topic is the only TopicDescription that can be used for publications and therefore associated to a DataWriter.
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable and
/// get_status_condition may return the value NOT_ENABLED.
pub struct Topic<'topic, T: DDSType>(Option<TopicImpl<'topic, T>>);

impl<'topic, T:DDSType> Topic<'topic, T> {
    /// This method allows the application to retrieve the INCONSISTENT_TOPIC status of the Topic.
    /// Each DomainEntity has a set of relevant communication statuses. A change of status causes the corresponding Listener to be
    /// invoked and can also be monitored by means of the associated StatusCondition.
    /// The complete list of communication status, their values, and the DomainEntities they apply to is provided in 2.2.4.1,
    /// Communication Status.
    pub fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> ReturnCode<()> {
        // TopicImpl::get_inconsistent_topic_status(&self.0, status)
        todo!()
    }

    // //////////// From here on are the functions that do not belong to the official API
    pub(crate) fn new(parent_participant: &'topic DomainParticipant, topic_name: String) -> Self {
        Self(Some(TopicImpl{
            parent_participant,
            topic_name,
            topic_data: std::marker::PhantomData
        }))
    }

    pub(crate) fn delete(&mut self) -> ReturnCode<()>{
        self.0 = None;
        Ok(())
    }

    fn topic_impl(&self) -> ReturnCode<&TopicImpl<T>> {
        match &self.0 {
            Some(topicimpl) => Ok(topicimpl),
            None => Err(ReturnCodes::AlreadyDeleted("Topic already deleted")),
        }
    }
}

impl<'topic, T:DDSType> TopicDescription for Topic<'topic, T> {
    fn get_participant(&self) -> ReturnCode<&DomainParticipant> {
        Ok(self.topic_impl()?.parent_participant)
    }

    fn get_type_name(&self) -> ReturnCode<&str> {
        self.topic_impl()?; // Just to check if it hasn't been deleted already
        Ok(T::type_name())
    }

    fn get_name(&self) -> ReturnCode<&String> {
        Ok(&self.topic_impl()?.topic_name)
    }
}

impl<'topic, T:DDSType> Entity for Topic<'topic, T> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener<T>>;

    fn set_qos(&self, _qos_list: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self, _qos_list: &mut Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self, ) -> Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self, ) -> crate::infrastructure::entity::StatusCondition {
        todo!()
    }

    fn get_status_changes(&self, ) -> StatusMask {
        todo!()
    }

    fn enable(&self, ) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        todo!()
    }
}

impl<'topic, T:DDSType> DomainEntity for Topic<'topic, T>{}

impl<'topic, T:DDSType> Drop for Topic<'topic, T> {
    fn drop(&mut self) {
        if let Some(topic_impl) = &self.0 {
            topic_impl.parent_participant.delete_topic(self).ok();
        };
    }
}

