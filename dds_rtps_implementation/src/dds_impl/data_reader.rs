use rust_dds_api::{infrastructure::qos::DataReaderQos, return_type::DDSResult};

use crate::rtps_impl::rtps_reader_impl::RtpsReaderImpl;

pub struct DataReader {
    rtps_reader: RtpsReaderImpl,
    qos: DataReaderQos,
}

impl DataReader {
    pub fn new(rtps_reader: RtpsReaderImpl, qos: DataReaderQos) -> Self {
        Self { rtps_reader, qos }
    }

    /// Get a reference to the data reader storage's reader.
    pub fn rtps_reader(&self) -> &RtpsReaderImpl {
        &self.rtps_reader
    }

    /// Get a mutable reference to the data reader storage's reader.
    pub fn rtps_reader_mut(&mut self) -> &mut RtpsReaderImpl {
        &mut self.rtps_reader
    }

    pub fn set_qos(&mut self, qos: Option<DataReaderQos>) -> DDSResult<()> {
        self.qos = qos.unwrap_or_default();
        Ok(())
    }

    pub fn get_qos(&self) -> DDSResult<&DataReaderQos> {
        Ok(&self.qos)
    }
}
