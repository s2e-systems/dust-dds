use rust_dds_api::dcps_psm::{InstanceStateKind, ViewStateKind};
use rust_rtps_pim::{
    behavior::{
        stateful_writer_behavior::{
            BestEffortStatefulWriterBehavior, ReliableStatefulWriterBehavior,
            StatefulWriterBehavior,
        },
        types::Duration,
        writer::{
            reader_proxy::RtpsReaderProxyAttributes,
            stateful_writer::{RtpsStatefulWriterConstructor, RtpsStatefulWriterOperations},
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    messages::types::Count,
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        history_cache::RtpsHistoryCacheConstructor,
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
};
use rust_rtps_psm::messages::submessage_elements::ParameterOwned;

use crate::utils::clock::{StdTimer, Timer};

use super::{
    rtps_reader_proxy_impl::RtpsReaderProxyAttributesImpl,
    rtps_writer_history_cache_impl::{WriterCacheChange, WriterHistoryCache},
};

pub struct RtpsStatefulWriterImpl {
    guid: Guid,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    data_max_size_serialized: Option<i32>,
    writer_cache: WriterHistoryCache,
    matched_readers: Vec<RtpsReaderProxyAttributesImpl>,
    heartbeat_timer: StdTimer,
    heartbeat_count: Count,
}

impl RtpsStatefulWriterOperations for RtpsStatefulWriterImpl {
    type ReaderProxyType = RtpsReaderProxyAttributesImpl;

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType) {
        self.matched_readers.push(a_reader_proxy)
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid) {
        self.matched_readers
            .retain(|x| x.remote_reader_guid() != reader_proxy_guid);
    }

    fn matched_reader_lookup(&self, a_reader_guid: &Guid) -> Option<&Self::ReaderProxyType> {
        self.matched_readers
            .iter()
            .find(|&x| x.remote_reader_guid() == a_reader_guid)
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}
impl RtpsStatefulWriterConstructor for RtpsStatefulWriterImpl {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        Self {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list: unicast_locator_list.iter().cloned().collect(),
            multicast_locator_list: multicast_locator_list.iter().cloned().collect(),
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: 0,
            data_max_size_serialized,
            writer_cache: WriterHistoryCache::new(),
            matched_readers: Vec::new(),
            heartbeat_timer: StdTimer::new(),
            heartbeat_count: Count(0),
        }
    }
}

impl RtpsEntityAttributes for RtpsStatefulWriterImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}

impl RtpsEndpointAttributes for RtpsStatefulWriterImpl {
    fn topic_kind(&self) -> &TopicKind {
        &self.topic_kind
    }

    fn reliability_level(&self) -> &ReliabilityKind {
        &self.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_ref()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.as_ref()
    }
}

impl RtpsWriterAttributes for RtpsStatefulWriterImpl {
    type WriterHistoryCacheType = WriterHistoryCache;

    fn push_mode(&self) -> &bool {
        &self.push_mode
    }

    fn heartbeat_period(&self) -> &Duration {
        &self.heartbeat_period
    }

    fn nack_response_delay(&self) -> &Duration {
        &self.nack_response_delay
    }

    fn nack_suppression_duration(&self) -> &Duration {
        &self.nack_suppression_duration
    }

    fn last_change_sequence_number(&self) -> &SequenceNumber {
        &self.last_change_sequence_number
    }

    fn data_max_size_serialized(&self) -> &Option<i32> {
        &self.data_max_size_serialized
    }

    fn writer_cache(&mut self) -> &mut Self::WriterHistoryCacheType {
        &mut self.writer_cache
    }
}

impl RtpsWriterOperations for RtpsStatefulWriterImpl {
    type DataType = Vec<u8>;
    type ParameterListType = Vec<ParameterOwned>;
    type CacheChangeType = WriterCacheChange;
    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Self::DataType,
        _inline_qos: Self::ParameterListType,
        handle: InstanceHandle,
    ) -> Self::CacheChangeType {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        WriterCacheChange {
            kind,
            writer_guid: self.guid,
            sequence_number: self.last_change_sequence_number,
            instance_handle: handle,
            data,
            _source_timestamp: None,
            _view_state_kind: ViewStateKind::New,
            _instance_state_kind: InstanceStateKind::Alive,
            inline_qos: vec![],
        }
    }
}

pub struct RtpsReaderProxyIterator<'a> {
    reader_proxy_iterator: std::slice::IterMut<'a, RtpsReaderProxyAttributesImpl>,
    writer_cache: &'a WriterHistoryCache,
    last_change_sequence_number: &'a SequenceNumber,
    reliability_level: &'a ReliabilityKind,
    writer_guid: &'a Guid,
    heartbeat_count: &'a Count,
    after_heartbeat_period: bool,
}

impl<'a> Iterator for RtpsReaderProxyIterator<'a> {
    type Item = StatefulWriterBehavior<'a, RtpsReaderProxyAttributesImpl, WriterHistoryCache>;

    fn next(&mut self) -> Option<Self::Item> {
        let reader_proxy = self.reader_proxy_iterator.next()?;
        match self.reliability_level {
            ReliabilityKind::BestEffort => Some(StatefulWriterBehavior::BestEffort(
                BestEffortStatefulWriterBehavior {
                    reader_proxy,
                    writer_cache: self.writer_cache,
                    last_change_sequence_number: self.last_change_sequence_number,
                },
            )),
            ReliabilityKind::Reliable => Some(StatefulWriterBehavior::Reliable(
                ReliableStatefulWriterBehavior {
                    reader_proxy,
                    writer_cache: self.writer_cache,
                    last_change_sequence_number: self.last_change_sequence_number,
                    writer_guid: self.writer_guid,
                    heartbeat_count: self.heartbeat_count,
                    after_heartbeat_period: self.after_heartbeat_period,
                },
            )),
        }
    }
}

impl<'a> IntoIterator for &'a mut RtpsStatefulWriterImpl {
    type Item = StatefulWriterBehavior<'a, RtpsReaderProxyAttributesImpl, WriterHistoryCache>;
    type IntoIter = RtpsReaderProxyIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let heartbeat_period_duration = std::time::Duration::new(
            self.heartbeat_period.seconds as u64,
            self.heartbeat_period.fraction,
        );

        let after_heartbeat_period = if self.heartbeat_timer.elapsed() > heartbeat_period_duration {
            self.heartbeat_count += Count(1);
            self.heartbeat_timer.reset();
            true
        } else {
            false
        };

        RtpsReaderProxyIterator {
            reader_proxy_iterator: self.matched_readers.iter_mut(),
            writer_cache: &self.writer_cache,
            last_change_sequence_number: &self.last_change_sequence_number,
            reliability_level: &self.reliability_level,
            writer_guid: &self.guid,
            heartbeat_count: &self.heartbeat_count,
            after_heartbeat_period,
        }
    }
}
