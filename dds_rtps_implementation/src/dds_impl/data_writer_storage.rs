use rust_dds_api::infrastructure::qos::DataWriterQos;

use crate::rtps_impl::rtps_writer_impl::RtpsWriterImpl;

pub struct DataWriterStorage {
    qos: DataWriterQos,
    rtps_data_writer: RtpsWriterImpl,
}

impl DataWriterStorage {
    pub fn new(qos: DataWriterQos, rtps_data_writer: RtpsWriterImpl) -> Self {
        Self {
            qos,
            rtps_data_writer,
        }
    }

    /// Get a reference to the data writer storage's rtps data writer.
    pub fn rtps_data_writer(&self) -> &RtpsWriterImpl {
        &self.rtps_data_writer
    }

    /// Get a mutable reference to the data writer storage's rtps data writer.
    pub fn rtps_data_writer_mut(&mut self) -> &mut RtpsWriterImpl {
        &mut self.rtps_data_writer
    }
}
