use crate::types::DDSType;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;
pub struct RtpsDataWriterInner<T: DDSType>{
    marker: std::marker::PhantomData<T>
}

impl<T: DDSType> Default for RtpsDataWriterInner<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData
        }
    }
}

pub type RtpsDataWriter<'a, T> = RtpsObjectReference<'a, RtpsDataWriterInner<T>>;

impl<'a, T:DDSType> RtpsDataWriter<'a,T> {

}