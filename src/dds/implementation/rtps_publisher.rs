use crate::types::DDSType;
use crate::dds::implementation::rtps_data_writer::RtpsDataWriter;
use crate::dds::implementation::rtps_object::RtpsObject;
#[derive(Default)]
pub struct RtpsPublisherInner;

pub type RtpsPublisher<'a> = RtpsObject<RtpsPublisherInner>;


impl<'a> RtpsPublisher<'a> {
    pub fn create_datawriter<T:DDSType>(&self) -> Option<&RtpsDataWriter<T>> {
        todo!()
    }
}