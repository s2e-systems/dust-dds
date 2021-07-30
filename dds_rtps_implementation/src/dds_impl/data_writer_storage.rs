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
}
