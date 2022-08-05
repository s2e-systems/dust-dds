use dds_transport::types::Locator;

use crate::{
    dcps_psm::{Duration, LENGTH_UNLIMITED},
    infrastructure::{qos::DataReaderQos, qos_policy::HistoryQosPolicyKind},
    return_type::DdsResult,
};

use super::{
    endpoint::RtpsEndpoint,
    reader_cache_change::RtpsReaderCacheChange,
    types::{Guid, ReliabilityKind, SequenceNumber, TopicKind},
};

struct ReaderHistoryCache {
    changes: Vec<RtpsReaderCacheChange>,
}

impl ReaderHistoryCache {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
}

pub struct RtpsReader {
    endpoint: RtpsEndpoint,
    heartbeat_response_delay: Duration,
    heartbeat_suppression_duration: Duration,
    reader_cache: ReaderHistoryCache,
    expects_inline_qos: bool,
    qos: DataReaderQos,
}

impl RtpsReader {
    pub fn new(
        endpoint: RtpsEndpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
        qos: DataReaderQos,
    ) -> Self {
        Self {
            endpoint,
            heartbeat_response_delay,
            heartbeat_suppression_duration,
            reader_cache: ReaderHistoryCache::new(),
            expects_inline_qos,
            qos,
        }
    }
}

impl RtpsReader {
    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }
}

impl RtpsReader {
    pub fn topic_kind(&self) -> TopicKind {
        self.endpoint.topic_kind()
    }

    pub fn reliability_level(&self) -> ReliabilityKind {
        self.endpoint.reliability_level()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}

impl RtpsReader {
    pub fn heartbeat_response_delay(&self) -> Duration {
        self.heartbeat_response_delay
    }

    pub fn heartbeat_suppression_duration(&self) -> Duration {
        self.heartbeat_suppression_duration
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
}

impl RtpsReader {
    pub fn changes(&self) -> &[RtpsReaderCacheChange] {
        self.reader_cache.changes.as_ref()
    }

    pub fn add_change(&mut self, change: RtpsReaderCacheChange) {
        if self.qos.history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS {
            let cache_len = self.reader_cache.changes.len() as i32;
            if cache_len > self.qos.history.depth {
                let mut seq_nums: Vec<_> = self
                    .reader_cache
                    .changes
                    .iter()
                    .map(|c| c.sequence_number())
                    .collect();
                seq_nums.sort_unstable();

                let to_delete =
                    &seq_nums[0..(cache_len as usize - self.qos.history.depth as usize)];
                self.remove_change(|c| to_delete.contains(&c.sequence_number()));
            }
        }

        if self.qos.resource_limits.max_samples == LENGTH_UNLIMITED
            || (self.reader_cache.changes.len() as i32) < self.qos.resource_limits.max_samples
        {
            self.reader_cache.changes.push(change);
        }
    }

    pub fn remove_change<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsReaderCacheChange) -> bool,
    {
        self.reader_cache.changes.retain(|cc| !f(cc));
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.reader_cache
            .changes
            .iter()
            .map(|cc| cc.sequence_number())
            .min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.reader_cache
            .changes
            .iter()
            .map(|cc| cc.sequence_number())
            .max()
    }
}

impl RtpsReader {
    pub fn get_qos(&self) -> &DataReaderQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }
}
