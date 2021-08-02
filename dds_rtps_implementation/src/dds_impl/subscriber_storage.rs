use rust_dds_api::infrastructure::qos::SubscriberQos;

use crate::utils::shared_object::RtpsShared;

use super::data_reader_storage::DataReaderStorage;

pub struct SubscriberStorage {
    qos: SubscriberQos,
    data_reader_storage_list: Vec<RtpsShared<DataReaderStorage>>,
}

impl SubscriberStorage {
    pub fn new(
        qos: SubscriberQos,
        data_reader_storage_list: Vec<RtpsShared<DataReaderStorage>>,
    ) -> Self {
        Self {
            qos,
            data_reader_storage_list,
        }
    }

    /// Get a reference to the subscriber storage's qos.
    pub fn qos(&self) -> &SubscriberQos {
        &self.qos
    }

    /// Get a reference to the subscriber storage's readers.
    pub fn readers(&self) -> &[RtpsShared<DataReaderStorage>] {
        self.data_reader_storage_list.as_slice()
    }
}
