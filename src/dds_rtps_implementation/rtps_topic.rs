use crate::types::DDSType;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;

pub struct RtpsTopicInner<T: DDSType>{
    marker: std::marker::PhantomData<T>
}

impl<T: DDSType> Default for RtpsTopicInner<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData
        }
    }
}

pub type RtpsTopic<'a, T> = RtpsObjectReference<'a, RtpsTopicInner<T>>;

impl<'a, T:DDSType> RtpsTopic<'a,T> {

}