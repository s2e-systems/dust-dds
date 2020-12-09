use crate::types::DDSType;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;
pub struct RtpsDataReaderInner<T: DDSType>{
    marker: std::marker::PhantomData<T>
}

impl<T: DDSType> Default for RtpsDataReaderInner<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData
        }
    }
}

pub type RtpsDataReader<'a, T> = RtpsObjectReference<'a, RtpsDataReaderInner<T>>;

impl<'a, T:DDSType> RtpsDataReader<'a,T> {

}