use crate::types::DDSType;
use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriter;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;

#[derive(Default)]
pub struct RtpsPublisherInner;

pub type RtpsPublisher<'a> = RtpsObjectReference<'a, RtpsPublisherInner>;

impl<'a> RtpsPublisher<'a> {
    pub fn create_datawriter<T:DDSType>(&self) -> Option<RtpsDataWriter<T>> {
        todo!()
    }
}