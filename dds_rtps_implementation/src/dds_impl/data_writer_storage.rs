use rust_dds_api::infrastructure::qos::DataWriterQos;

use crate::rtps_impl::rtps_writer_impl::RTPSWriterImpl;

pub struct DataWriterStorage {
    qos: DataWriterQos,
    rtps_data_writer: RTPSWriterImpl,
}

impl DataWriterStorage {
    pub fn new(qos: DataWriterQos, rtps_data_writer: RTPSWriterImpl) -> Self {
        Self {
            qos,
            rtps_data_writer,
        }
    }

    /// Get a reference to the data writer storage's rtps data writer.
    pub fn rtps_data_writer(&self) -> &RTPSWriterImpl {
        &self.rtps_data_writer
    }

    /// Get a mutable reference to the data writer storage's rtps data writer.
    pub fn rtps_data_writer_mut(&mut self) -> &mut RTPSWriterImpl {
        &mut self.rtps_data_writer
    }
}
