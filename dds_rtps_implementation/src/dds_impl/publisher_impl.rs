use rust_dds_api::infrastructure::qos::{DataWriterQos, PublisherQos};

use crate::{rtps_impl::rtps_group_impl::RtpsGroupImpl, utils::shared_object::RtpsShared};

use super::data_writer_impl::DataWriterImpl;

pub struct PublisherImpl {
    pub qos: PublisherQos,
    pub rtps_group: RtpsGroupImpl,
    pub data_writer_storage_list: Vec<RtpsShared<DataWriterImpl>>,
    pub user_defined_data_writer_counter: u8,
    pub default_datawriter_qos: DataWriterQos,
}

impl PublisherImpl {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroupImpl,
        data_writer_storage_list: Vec<RtpsShared<DataWriterImpl>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_writer_storage_list,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
        }
    }

    /// Get a reference to the publisher storage's data writer storage list.
    pub fn data_writer_storage_list(&self) -> &[RtpsShared<DataWriterImpl>] {
        self.data_writer_storage_list.as_slice()
    }
}
