use rust_dds_api::infrastructure::qos::DataWriterQos;

pub struct WriterFactory<'a, PSM: rust_rtps_pim::PIM> {
    guid_prefix: PSM::GuidPrefix,
    datawriter_counter: u8,
    default_datawriter_qos: DataWriterQos<'a>,
}

impl<'a, PSM: rust_rtps_pim::PIM> WriterFactory<'a, PSM> {
    pub fn new(guid_prefix: PSM::GuidPrefix) -> Self {
        Self {
            guid_prefix,
            datawriter_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
        }
    }

    pub fn create_datawriter(&mut self) {

    }

    pub fn set_default_datawriter_qos(&mut self, _default_datawriter_qos: DataWriterQos<'a>) {

    }
}
