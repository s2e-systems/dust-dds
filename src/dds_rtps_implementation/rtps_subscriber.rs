use crate::dds_rtps_implementation::rtps_data_reader::RtpsDataReader;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;
use crate::types::{DDSType, ReturnCode};

#[derive(Default)]
pub struct RtpsSubscriberInner;

pub type RtpsSubscriber<'a> = RtpsObjectReference<'a, RtpsSubscriberInner>;

impl<'a> RtpsSubscriber<'a> {
    pub fn create_datareader<T: DDSType>(&self) -> Option<RtpsDataReader<T>> {
        todo!()
    }

    pub fn delete_datareader<T: DDSType>(&self, a_datareader: &RtpsDataReader<T>) -> ReturnCode<()> {
        todo!()
    }
}
