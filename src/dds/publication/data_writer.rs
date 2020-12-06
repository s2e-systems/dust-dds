use crate::types::DDSType;
use crate::dds::implementation::rtps_data_writer::RtpsDataWriter;

pub struct DataWriter<'a, T: DDSType>(pub &'a RtpsDataWriter<T>);

impl<'a, T: DDSType> std::ops::Deref for DataWriter<'a, T> {
    type Target = RtpsDataWriter<T>;

    fn deref(&self) -> &Self::Target {
        self.0
    }    
}