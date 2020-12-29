use crate::dds::domain::domain_participant::DomainParticipant;
use crate::dds::infrastructure::entity::{Entity, StatusCondition};
use crate::dds::infrastructure::qos::TopicQos;
use crate::dds::infrastructure::status::StatusMask;
use crate::utils::validatable::{Validatable, Ref};
use crate::dds::topic::topic_description::TopicDescription;
use crate::dds::topic::topic_listener::TopicListener;
use crate::rtps;
use crate::rtps::types::GUID;
use crate::types::{DDSType, InstanceHandle, ReturnCode, ReturnCodes, TopicKind };
use std::sync::{Arc, Mutex};
use std::any::Any;

pub struct RtpsTopic<T: DDSType> {
    pub rtps_entity: rtps::structure::Entity,
    pub topic_name: String,
    pub type_name: &'static str,
    pub topic_kind: TopicKind,
    pub qos: Mutex<TopicQos>,
    pub listener: Option<Box<dyn TopicListener<T>>>,
    pub status_mask: StatusMask,
}

impl<T: DDSType> RtpsTopic<T> {
    pub fn new(
        guid: GUID,
        topic_name: String,
        qos: TopicQos,
        listener: Option<Box<dyn TopicListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            rtps_entity: rtps::structure::Entity { guid },
            topic_name,
            type_name: T::type_name(),
            topic_kind: T::topic_kind(),
            qos: Mutex::new(qos),
            listener,
            status_mask,
        }
    }
}

pub trait AnyRtpsTopic: Send + Sync {
    fn rtps_entity(&self) -> &rtps::structure::Entity;
    fn topic_kind(&self) -> TopicKind;
    fn type_name(&self) -> &'static str;
    fn topic_name(&self) -> &String;
    fn qos(&self) -> &Mutex<TopicQos>;
    fn as_any(&self) -> &dyn Any;
}

impl<T: DDSType + Sized> AnyRtpsTopic for RtpsTopic<T> {
    fn rtps_entity(&self) -> &rtps::structure::Entity {
        &self.rtps_entity
    }

    fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    fn type_name(&self) -> &'static str {
        &self.type_name
    }

    fn topic_name(&self) -> &String {
        &self.topic_name
    }

    fn qos(&self) -> &Mutex<TopicQos> {
        &self.qos
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<'a> Ref<'a, Validatable<Arc<dyn AnyRtpsTopic>>> {
    pub fn value_as<U: 'static>(&self) -> ReturnCode<&U> {
        self.value()?.as_ref().as_any().downcast_ref::<U>().ok_or(ReturnCodes::Error)
    }
}

/// Topic is the most basic description of the data to be published and subscribed.
/// A Topic is identified by its name, which must be unique in the whole Domain. In addition (by virtue of extending
/// TopicDescription) it fully specifies the type of the data that can be communicated when publishing or subscribing to the Topic.
/// Topic is the only TopicDescription that can be used for publications and therefore associated to a DataWriter.
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable and
/// get_status_condition may return the value NOT_ENABLED.
pub struct Topic<'a, T: DDSType> {
    pub(crate) parent_participant: &'a DomainParticipant,
    pub(crate) rtps_topic: Ref<'a, Validatable<Arc<dyn AnyRtpsTopic>>>,
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
        Ok(self.rtps_topic.value()?.type_name())
    }

    fn get_name(&self) -> ReturnCode<String> {
        Ok(self.rtps_topic.value()?.topic_name().clone())
    }
}

impl<'a, T: DDSType> Entity for Topic<'a, T> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener<T>>;

    fn set_qos(&self, qos: Self::Qos) -> ReturnCode<()> {
        qos.is_consistent()?;
        *self.rtps_topic.value()?.qos().lock().unwrap() = qos;
        // discovery.update_topic(topic)?;
        Ok(())
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        Ok(self.rtps_topic.value()?.qos().lock().unwrap().clone())
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
        todo!()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        Ok(self.rtps_topic.value()?.rtps_entity().guid.into())
    }
}
