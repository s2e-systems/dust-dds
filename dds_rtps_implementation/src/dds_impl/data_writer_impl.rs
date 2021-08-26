use rust_dds_api::{dcps_psm::InstanceHandle, infrastructure::qos::DataWriterQos, return_type::DDSResult};
use rust_rtps_pim::{behavior::writer::writer::{RtpsWriter, RtpsWriterOperations}, structure::{RtpsHistoryCache, types::ChangeKind}};

use crate::{dds_type::DDSType, rtps_impl::rtps_writer_impl::RtpsWriterImpl};

pub struct DataWriterImpl {
    qos: DataWriterQos,
    rtps_data_writer: RtpsWriterImpl,
}

impl DataWriterImpl {
    pub fn new(qos: DataWriterQos, rtps_data_writer: RtpsWriterImpl) -> Self {
        Self {
            qos,
            rtps_data_writer,
        }
    }

    /// Get a reference to the data writer storage's rtps data writer.
    pub fn rtps_data_writer(&self) -> &RtpsWriterImpl {
        &self.rtps_data_writer
    }

    /// Get a mutable reference to the data writer storage's rtps data writer.
    pub fn rtps_data_writer_mut(&mut self) -> &mut RtpsWriterImpl {
        &mut self.rtps_data_writer
    }

    pub fn set_qos(&mut self, qos: Option<DataWriterQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> &DataWriterQos {
        &self.qos
    }



    pub fn write_w_timestamp<T: DDSType + 'static>(
        &mut self,
        data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        let data = cdr::serialize::<_, _, cdr::CdrLe>(&data, cdr::Infinite).unwrap();
        let change = self
            .rtps_data_writer
            .new_change(ChangeKind::Alive, data.as_slice(), &[], 0);
        let writer_cache = self.rtps_data_writer.writer_cache_mut();
        let time = rust_rtps_pim::messages::types::Time(0);
        writer_cache.set_source_timestamp(Some(time));
        writer_cache.add_change(&change);
        Ok(())
    }
}