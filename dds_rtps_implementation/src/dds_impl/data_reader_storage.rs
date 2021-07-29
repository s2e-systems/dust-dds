use rust_dds_api::{
    dcps_psm::{InstanceStateKind, SampleStateKind, Time, ViewStateKind},
    infrastructure::{qos::DataReaderQos, sample_info::SampleInfo},
    return_type::DDSResult,
};
use rust_rtps_pim::{behavior::reader::reader::RTPSReader, structure::RTPSHistoryCache};
use serde::Deserialize;

use crate::rtps_impl::rtps_reader_impl::RTPSReaderImpl;

pub struct DataReaderStorage {
    reader: RTPSReaderImpl,
    qos: DataReaderQos,
}

impl DataReaderStorage {
    pub fn read<T>(
        &mut self,
        max_samples: i32,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<Vec<(T, SampleInfo)>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut samples = Vec::new();
        let reader_cache = self.reader.reader_cache();
        if let Some(seq_num_min) = reader_cache.get_seq_num_min() {
            let seq_num_max = reader_cache.get_seq_num_max().unwrap();
            for seq_num in seq_num_min..=seq_num_max {
                let cc1 = reader_cache.get_change(&(seq_num as i64)).unwrap();
                let data = cc1.data_value();
                let value = rust_serde_cdr::deserializer::from_bytes(data).unwrap();
                let sample_info = SampleInfo {
                    sample_state: SampleStateKind::NotRead,
                    view_state: ViewStateKind::New,
                    instance_state: InstanceStateKind::Alive,
                    disposed_generation_count: 0,
                    no_writers_generation_count: 0,
                    sample_rank: 0,
                    generation_rank: 0,
                    absolute_generation_rank: 0,
                    source_timestamp: Time { sec: 0, nanosec: 0 },
                    instance_handle: 0,
                    publication_handle: 0,
                    valid_data: true,
                };
                samples.push((value, sample_info));
                if samples.len() >= max_samples as usize {
                    break;
                }
            }
        }
        Ok(samples)
    }

    pub fn set_qos(&mut self, _qos: DataReaderQos) -> DDSResult<()> {
        todo!()
    }

    pub fn qos(&self) -> &DataReaderQos {
        &self.qos
    }
}
