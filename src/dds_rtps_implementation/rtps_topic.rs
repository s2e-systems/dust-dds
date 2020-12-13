use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use std::cell::Ref;

pub struct RtpsTopicInner{
}

impl Default for RtpsTopicInner {
    fn default() -> Self {
        Self {
        }
    }
}

pub type RtpsTopic<'a> = Ref<'a, RtpsObject<RtpsTopicInner>>;

impl RtpsObject<RtpsTopicInner> {}
