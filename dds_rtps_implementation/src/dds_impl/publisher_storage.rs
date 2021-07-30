use rust_dds_api::infrastructure::qos::PublisherQos;

use crate::utils::shared_object::RtpsShared;

use super::data_writer_storage::DataWriterStorage;

pub struct PublisherStorage {
    qos: PublisherQos,
    data_writer_storage_list: Vec<RtpsShared<DataWriterStorage>>,
}

impl PublisherStorage {
    pub fn new(
        qos: PublisherQos,
        data_writer_storage_list: Vec<RtpsShared<DataWriterStorage>>,
    ) -> Self {
        Self {
            qos,
            data_writer_storage_list,
        }
    }
}
