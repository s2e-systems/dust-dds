use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
};

use crate::rtps_impl::rtps_writer_impl::RtpsWriterImpl;

pub struct WriterFactory {
    _guid_prefix: rust_rtps_pim::structure::types::GuidPrefix,
    _datawriter_counter: u8,
}

impl WriterFactory {
    pub fn new(guid_prefix: rust_rtps_pim::structure::types::GuidPrefix) -> Self {
        Self {
            _guid_prefix: guid_prefix,
            _datawriter_counter: 0,
        }
    }

    pub fn create_datawriter<'a, T>(
        &mut self,
        _qos: DataWriterQos,
        _a_listener: Option<&'a (dyn DataWriterListener<DataPIM = T> + 'a)>,
        _mask: StatusMask,
    ) -> RtpsWriterImpl {
        todo!()
    }
}
