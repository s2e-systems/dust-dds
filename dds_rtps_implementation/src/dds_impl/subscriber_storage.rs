use rust_dds_api::infrastructure::qos::SubscriberQos;

use crate::utils::shared_object::RtpsShared;

use super::data_reader_storage::DataReaderStorage;

pub struct SubscriberStorage {
    readers: Vec<RtpsShared<DataReaderStorage>>,
    qos: SubscriberQos,
}

impl SubscriberStorage {
    pub fn new(readers: Vec<RtpsShared<DataReaderStorage>>, qos: SubscriberQos) -> Self {
        Self { readers, qos }
    }

    /// Get a reference to the subscriber storage's qos.
    pub fn qos(&self) -> &SubscriberQos {
        &self.qos
    }

    /// Get a reference to the subscriber storage's readers.
    pub fn readers(&self) -> &[RtpsShared<DataReaderStorage>] {
        self.readers.as_slice()
    }
}
