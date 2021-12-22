use rust_dds_api::dcps_psm::{InstanceStateKind, ViewStateKind};
use rust_rtps_pim::{
    behavior::{
        stateless_writer_behavior::{
            BestEffortStatelessWriterBehavior, ReliableStatelessWriterBehavior,
            StatelessWriterBehavior,
        },
        types::Duration,
        writer::{
            reader_locator::{RtpsReaderLocator, RtpsReaderLocatorAttributes},
            stateless_writer::{RtpsStatelessWriterConstructor, RtpsStatelessWriterOperations},
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        history_cache::RtpsHistoryCacheConstructor,
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
};
use rust_rtps_psm::messages::submessage_elements::Parameter;

use super::{
    rtps_reader_locator_impl::RtpsReaderLocatorImpl,
    rtps_writer_history_cache_impl::{WriterCacheChange, WriterHistoryCache},
};

pub struct RtpsStatelessWriterImpl {
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
    reader_locators: Vec<RtpsReaderLocatorImpl>,
}

pub struct RtpsReaderLocatorIterator<'a> {
    reader_locator_iterator: std::slice::IterMut<'a, RtpsReaderLocatorImpl>,
    writer_cache: &'a WriterHistoryCache,
    last_change_sequence_number: &'a SequenceNumber,
    reliability_level: &'a ReliabilityKind,
    writer_guid: &'a Guid,
}

impl<'a> Iterator for RtpsReaderLocatorIterator<'a> {
    type Item = StatelessWriterBehavior<'a, RtpsReaderLocatorImpl, WriterHistoryCache>;

    fn next(&mut self) -> Option<Self::Item> {
        let reader_locator = self.reader_locator_iterator.next()?;
        match self.reliability_level {
            ReliabilityKind::BestEffort => Some(StatelessWriterBehavior::BestEffort(
                BestEffortStatelessWriterBehavior {
                    reader_locator,
                    writer_cache: self.writer_cache,
                    last_change_sequence_number: self.last_change_sequence_number,
                },
            )),
            ReliabilityKind::Reliable => Some(StatelessWriterBehavior::Reliable(
                ReliableStatelessWriterBehavior {
                    reader_locator,
                    writer_cache: self.writer_cache,
                    last_change_sequence_number: self.last_change_sequence_number,
                    writer_guid: self.writer_guid,
                },
            )),
        }
    }
}

impl<'a> IntoIterator for &'a mut RtpsStatelessWriterImpl {
    type Item = StatelessWriterBehavior<'a, RtpsReaderLocatorImpl, WriterHistoryCache>;

    type IntoIter = RtpsReaderLocatorIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RtpsReaderLocatorIterator {
            reader_locator_iterator: self.reader_locators.iter_mut(),
            writer_cache: &self.writer_cache,
            last_change_sequence_number: &self.last_change_sequence_number,
            reliability_level: &self.reliability_level,
            writer_guid: &self.guid,
        }
    }
}

impl RtpsStatelessWriterConstructor for RtpsStatelessWriterImpl {
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
            reader_locators: Vec::new(),
        }
    }
}

impl RtpsEntityAttributes for RtpsStatelessWriterImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}

impl RtpsEndpointAttributes for RtpsStatelessWriterImpl {
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

impl RtpsWriterAttributes for RtpsStatelessWriterImpl {
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

impl RtpsStatelessWriterOperations for RtpsStatelessWriterImpl {
    fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator) {
        let reader_locator_impl = RtpsReaderLocatorImpl::new(a_locator);
        self.reader_locators.push(reader_locator_impl);
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| x.locator() != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

impl RtpsWriterOperations for RtpsStatelessWriterImpl {
    type DataType = Vec<u8>;
    type ParameterListType = Vec<Parameter<Vec<u8>>>;
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
        }
    }
}
