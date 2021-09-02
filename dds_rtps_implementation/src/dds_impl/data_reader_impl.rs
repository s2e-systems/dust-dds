use rust_dds_api::{infrastructure::qos::DataReaderQos, return_type::DDSResult};

use crate::rtps_impl::rtps_reader_impl::RtpsReaderImpl;

pub struct DataReaderImpl {
    rtps_reader: RtpsReaderImpl,
    qos: DataReaderQos,
}

impl DataReaderImpl {
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

// let shared_reader = self.reader.upgrade()?;
        // let mut reader = shared_reader.lock();
        // let reader_cache = reader.rtps_reader_mut().reader_cache_mut();
        // Ok(reader_cache
        //     .changes_mut()
        //     .iter()
        //     .map(|cc| {
        //         let data = cc.data();
        //         let value = cdr::deserialize(data).unwrap();
        //         let sample_info = SampleInfo {
        //             sample_state: *cc.sample_state_kind(),
        //             view_state: *cc.view_state_kind(),
        //             instance_state: *cc.instance_state_kind(),
        //             disposed_generation_count: 0,
        //             no_writers_generation_count: 0,
        //             sample_rank: 0,
        //             generation_rank: 0,
        //             absolute_generation_rank: 0,
        //             source_timestamp: Time { sec: 0, nanosec: 0 },
        //             instance_handle: 0,
        //             publication_handle: 0,
        //             valid_data: true,
        //         };
        //         (value, sample_info)
        //     })
        //     .collect())