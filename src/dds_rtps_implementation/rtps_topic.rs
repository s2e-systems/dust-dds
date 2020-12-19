use crate::dds_infrastructure::qos::TopicQos;
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::rtps::structure::Entity;
use crate::rtps::types::GUID;
use crate::rtps::types::constants::GUID_UNKNOWN;
use std::cell::Ref;

pub struct RtpsTopicInner {
    entity: Entity,
    topic_name: String,
    type_name: &'static str,
    qos: TopicQos,
}

impl Default for RtpsTopicInner {
    fn default() -> Self {
        Self {
            entity: Entity{ guid: GUID_UNKNOWN},
            topic_name: String::new(),
            type_name: "",
            qos: TopicQos::default(),
        }
    }
}

impl RtpsTopicInner {
    pub fn new(guid: GUID, topic_name: String, type_name: &'static str, qos: TopicQos) -> Self {
        Self {
            entity: Entity{ guid},
            topic_name,
            type_name,
            qos,
        }
    }
}

pub type RtpsTopic<'a> = Ref<'a, RtpsObject<RtpsTopicInner>>;

impl RtpsObject<RtpsTopicInner> {}
