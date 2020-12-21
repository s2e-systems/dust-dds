use crate::dds_infrastructure::qos::TopicQos;
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::rtps::structure::Entity;
use crate::rtps::types::GUID;
use crate::types::ReturnCode;
use std::sync::{Mutex, RwLockReadGuard};

pub struct RtpsTopicInner {
    entity: Entity,
    topic_name: String,
    type_name: &'static str,
    qos: Mutex<TopicQos>,
}

impl RtpsTopicInner {
    pub fn new(guid: GUID, topic_name: String, type_name: &'static str, qos: TopicQos) -> Self {
        Self {
            entity: Entity { guid },
            topic_name,
            type_name,
            qos: Mutex::new(qos),
        }
    }
}

pub type RtpsTopic<'a> = RwLockReadGuard<'a, RtpsObject<RtpsTopicInner>>;

impl RtpsObject<RtpsTopicInner> {
    pub fn get_name(&self) -> ReturnCode<String> {
        Ok(self.value()?.topic_name.clone())
    }
}
