use crate::dds::domain::domain_participant::DomainParticipant;
use crate::dds::topic::topic_description::TopicDescription;
use crate::dds_infrastructure::entity::{Entity, StatusCondition};
use crate::dds_infrastructure::qos::TopicQos;
use crate::dds_infrastructure::topic_listener::TopicListener;
use crate::dds_infrastructure::status::StatusMask;
use crate::dds_rtps_implementation::rtps_topic::RtpsTopic;
use crate::types::{DDSType, ReturnCode,InstanceHandle};

/// Topic is the most basic description of the data to be published and subscribed.
/// A Topic is identified by its name, which must be unique in the whole Domain. In addition (by virtue of extending
/// TopicDescription) it fully specifies the type of the data that can be communicated when publishing or subscribing to the Topic.
/// Topic is the only TopicDescription that can be used for publications and therefore associated to a DataWriter.
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable and
/// get_status_condition may return the value NOT_ENABLED.
pub struct Topic<'a, T: DDSType> {
    pub(crate) parent_participant: &'a DomainParticipant,
    pub(crate) rtps_topic: RtpsTopic<'a>,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<'a, T: DDSType> Topic<'a, T> {
    // /// This method allows the application to retrieve the INCONSISTENT_TOPIC status of the Topic.
    // /// Each DomainEntity has a set of relevant communication statuses. A change of status causes the corresponding Listener to be
    // /// invoked and can also be monitored by means of the associated StatusCondition.
    // /// The complete list of communication status, their values, and the DomainEntities they apply to is provided in 2.2.4.1,
    // /// Communication Status.
    // fn get_inconsistent_topic_status(
    //     &self, _status: &mut InconsistentTopicStatus,
    // ) -> ReturnCode<()>;
}

impl<'a, T: DDSType> TopicDescription for Topic<'a, T> {
    fn get_participant(&self) -> &DomainParticipant {
        self.parent_participant
    }

    fn get_type_name(&self) -> ReturnCode<&str> {
        Ok(T::type_name())
    }

    fn get_name(&self) -> ReturnCode<String> {
        self.rtps_topic.get_name()
    }
}

impl<'a, T:DDSType> Entity for Topic<'a, T> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener<T>>;

    fn set_qos(&self, _qos: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        self.rtps_topic.get_qos()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self) -> &Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> ReturnCode<()> {
        self.rtps_topic.enable()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        self.rtps_topic.get_instance_handle()
    }
}