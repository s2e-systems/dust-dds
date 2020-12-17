use crate::dds_infrastructure::qos::TopicQos;
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use std::cell::Ref;

pub struct RtpsTopicInner {
    name: String,
    type_name: String,
    qos: TopicQos,
}

impl Default for RtpsTopicInner {
    fn default() -> Self {
        Self {
            name: String::new(),
            type_name: String::new(),
            qos: TopicQos::default(),
        }
    }
}

pub type RtpsTopic<'a> = Ref<'a, RtpsObject<RtpsTopicInner>>;

impl RtpsObject<RtpsTopicInner> {}
