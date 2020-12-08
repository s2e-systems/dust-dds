use crate::types::DDSType;
use crate::dds::implementation::rtps_data_reader::RtpsDataReader;
pub struct RtpsSubscriber;

impl RtpsSubscriber {
    pub fn create_datareader<T: DDSType>(&self) -> Option<&RtpsDataReader<T>> {
        todo!()
    }
}