use rust_dds_api::infrastructure::qos::{DataReaderQos, SubscriberQos};

use crate::{rtps_impl::rtps_group_impl::RtpsGroupImpl, utils::shared_object::RtpsShared};

use super::data_reader_impl::DataReaderImpl;

pub struct SubscriberImpl {
    pub qos: SubscriberQos,
    pub rtps_group: RtpsGroupImpl,
    pub data_reader_storage_list: Vec<RtpsShared<DataReaderImpl>>,
    pub user_defined_data_reader_counter: u8,
    pub default_data_reader_qos: DataReaderQos,
}

impl SubscriberImpl {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroupImpl,
        data_reader_storage_list: Vec<RtpsShared<DataReaderImpl>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_reader_storage_list,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: DataReaderQos::default(),
        }
    }

    /// Get a reference to the subscriber storage's readers.
    pub fn readers(&self) -> &[RtpsShared<DataReaderImpl>] {
        self.data_reader_storage_list.as_slice()
    }
}