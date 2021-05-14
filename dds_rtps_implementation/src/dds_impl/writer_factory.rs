use rust_dds_api::{dcps_psm::StatusMask, infrastructure::qos::DataWriterQos, publication::data_writer_listener::DataWriterListener};

use crate::rtps_impl::rtps_writer_impl::RTPSWriterImpl;

pub struct WriterFactory<PSM: rust_rtps_pim::PIM> {
    guid_prefix: PSM::GuidPrefix,
    datawriter_counter: u8,
}

impl<PSM: rust_rtps_pim::PIM> WriterFactory<PSM> {
    pub fn new(guid_prefix: PSM::GuidPrefix) -> Self {
        Self {
            guid_prefix,
            datawriter_counter: 0,
        }
    }

    pub fn create_datawriter<'a, T>(
        &mut self,
        _qos: DataWriterQos,
        _a_listener: Option<&'a (dyn DataWriterListener<DataType = T> + 'a)>,
        _mask: StatusMask,
    ) -> RTPSWriterImpl<PSM> {
        todo!()
    }
}
