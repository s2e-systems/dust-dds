use super::{behavior_types::Duration, reader_proxy::RtpsReaderProxy};
use crate::transport::types::{Guid, ReliabilityKind, SequenceNumber};
use alloc::vec::Vec;

pub struct RtpsStatefulWriter {
    guid: Guid,
    matched_readers: Vec<RtpsReaderProxy>,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    data_max_size_serialized: usize,
}

impl RtpsStatefulWriter {
    pub fn new(guid: Guid, data_max_size_serialized: usize) -> Self {
        Self {
            guid,
            matched_readers: Vec::new(),
            heartbeat_period: Duration::from_millis(200),
            nack_response_delay: Duration::from_millis(0),
            data_max_size_serialized,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.data_max_size_serialized
    }

    pub fn is_change_acknowledged(&self, sequence_number: SequenceNumber) -> bool {
        !self
            .matched_readers
            .iter()
            .filter(|rp| rp.reliability() == ReliabilityKind::Reliable)
            .any(|rp| rp.unacked_changes(Some(sequence_number)))
    }

    pub fn time_until_next_repair(
        &self,
        now: crate::infrastructure::time::Time,
    ) -> Option<crate::infrastructure::time::Duration> {
        let mut min_time: Option<crate::infrastructure::time::Duration> = None;
        for rp in &self.matched_readers {
            if let Some(d) = rp.time_until_repair(now, self.nack_response_delay.into()) {
                min_time = Some(min_time.map_or(d, |min| min.min(d)));
            }
        }
        min_time
    }

    pub fn add_matched_reader(&mut self, reader_proxy: RtpsReaderProxy) {
        if let Some(rp) = self
            .matched_readers
            .iter_mut()
            .find(|rp| rp.remote_reader_guid() == reader_proxy.remote_reader_guid())
        {
            *rp = reader_proxy;
        } else {
            self.matched_readers.push(reader_proxy);
        }
    }

    pub fn delete_matched_reader(&mut self, reader_guid: Guid) {
        self.matched_readers
            .retain(|reader_proxy| reader_proxy.remote_reader_guid() != reader_guid);
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.heartbeat_period
    }

    pub fn matched_readers(&self) -> &[RtpsReaderProxy] {
        &self.matched_readers
    }

    pub fn matched_readers_mut(&mut self) -> &mut [RtpsReaderProxy] {
        &mut self.matched_readers
    }
}
