use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;

pub struct RtpsTopicInner{
}

impl Default for RtpsTopicInner {
    fn default() -> Self {
        Self {
        }
    }
}

pub type RtpsTopic<'a> = RtpsObjectReference<'a, RtpsTopicInner>;

impl<'a> RtpsTopic<'a> {}
