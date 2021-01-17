use std::{any::Any, sync::{Arc, Mutex}};

use crate::{dds::{
        infrastructure::{qos::TopicQos, status::StatusMask},
        topic::topic_listener::TopicListener,
    }, rtps::{self, types::GUID}, types::{DDSType, TopicKind}, utils::maybe_valid::MaybeValidRef};

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

pub trait AnyRtpsTopic: Any + Send + Sync {
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

pub type RtpsTopicRef<'a> = MaybeValidRef<'a, Arc<dyn AnyRtpsTopic>>;