use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crate::{dds::{
        infrastructure::{qos::TopicQos, status::StatusMask},
        topic::topic_listener::TopicListener,
    }, rtps::{self, types::GUID}, types::{DDSType, ReturnCode, ReturnCodes, TopicKind}, utils::{as_any::AsAny, maybe_valid::{MaybeValid, MaybeValidRef}}};

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

pub trait AnyRtpsTopic: AsAny + Send + Sync {
    fn rtps_entity(&self) -> &rtps::structure::Entity;
    fn topic_kind(&self) -> TopicKind;
    fn type_name(&self) -> &'static str;
    fn topic_name(&self) -> &String;
    fn qos(&self) -> &Mutex<TopicQos>;
}

impl<T: DDSType> AnyRtpsTopic for RtpsTopic<T> {
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
}

impl<T:DDSType> AsAny for RtpsTopic<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub type RtpsAnyTopicRef<'a> = MaybeValidRef<'a, Arc<dyn AnyRtpsTopic>>;

impl<'a> RtpsAnyTopicRef<'a> {
    pub fn get(&self) -> ReturnCode<&Arc<dyn AnyRtpsTopic>> {
        MaybeValid::get(self).ok_or(ReturnCodes::AlreadyDeleted)
    }

    pub fn get_as<U: DDSType>(&self) -> ReturnCode<&RtpsTopic<U>> {
        self.get()?
            .as_ref()
            .as_any()
            .downcast_ref()
            .ok_or(ReturnCodes::Error)
    }

    pub fn delete(&self) -> ReturnCode<()>{
        let rtps_topic = self.get()?;
        if Arc::strong_count(rtps_topic) == 1 {
            MaybeValid::delete(self);
            Ok(())
        } else {
            Err(ReturnCodes::PreconditionNotMet(
                "Topic still attached to some data reader or data writer",
            ))
        }
    }
}
