use std::sync::Weak;

use crate::types::ReturnCode;
use crate::infrastructure::status::InconsistentTopicStatus;
use crate::infrastructure::entity::Entity;
use crate::infrastructure::entity::DomainEntity;
use crate::topic::topic_listener::TopicListener;
use crate::topic::topic_description::TopicDescription;
use crate::topic::qos::TopicQos;
use crate::domain::domain_participant::DomainParticipant;

use crate::implementation::topic_impl::TopicImpl;

/// Topic is the most basic description of the data to be published and subscribed.
/// A Topic is identified by its name, which must be unique in the whole Domain. In addition (by virtue of extending
/// TopicDescription) it fully specifies the type of the data that can be communicated when publishing or subscribing to the Topic.
/// Topic is the only TopicDescription that can be used for publications and therefore associated to a DataWriter.
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable and
/// get_status_condition may return the value NOT_ENABLED.
pub struct Topic(pub(crate) Weak<TopicImpl>);

impl Topic {
    /// This method allows the application to retrieve the INCONSISTENT_TOPIC status of the Topic.
    /// Each DomainEntity has a set of relevant communication statuses. A change of status causes the corresponding Listener to be
    /// invoked and can also be monitored by means of the associated StatusCondition.
    /// The complete list of communication status, their values, and the DomainEntities they apply to is provided in 2.2.4.1,
    /// Communication Status.
    pub fn get_inconsistent_topic_status(
        &self,
        status: &mut InconsistentTopicStatus,
    ) -> ReturnCode<()> {
        TopicImpl::get_inconsistent_topic_status(&self.0, status)
    }
}

impl TopicDescription for Topic {
    fn get_participant(&self) -> Option<DomainParticipant> {
        TopicImpl::get_participant(&self.0)
    }

    fn get_type_name(&self) -> Option<String> {
        TopicImpl::get_type_name(&self.0)
    }

    fn get_name(&self) -> Option<String> {
        TopicImpl::get_name(&self.0)
    }
}

impl Entity for Topic {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener>;

    fn set_qos(&self, _qos_list: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self, _qos_list: &mut Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: &[crate::types::StatusKind]) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self, ) -> Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self, ) -> crate::infrastructure::entity::StatusCondition {
        todo!()
    }

    fn get_status_changes(&self, ) -> crate::types::StatusKind {
        todo!()
    }

    fn enable(&self, ) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self, ) -> crate::types::InstanceHandle {
        todo!()
    }
}

impl DomainEntity for Topic{}

// impl Drop for Topic {
//     fn drop(&mut self) {
//         if let Some(parent_participant) = self.get_participant() {
//             parent_participant.delete_topic(self);
//         }
//     }
// }

