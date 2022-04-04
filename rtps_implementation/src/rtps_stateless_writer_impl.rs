use rtps_pim::{
    behavior::{
        types::Duration,
        writer::{
            stateless_writer::{
                RtpsStatelessWriterAttributes, RtpsStatelessWriterConstructor,
                RtpsStatelessWriterOperations,
            },
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    messages::types::Count,
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
};

use crate::{
    rtps_reader_locator_impl::RtpsReaderLocatorOperationsImpl, utils::clock::TimerConstructor,
};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl,
    rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
    rtps_reader_locator_impl::RtpsReaderLocatorAttributesImpl,
    rtps_writer_impl::RtpsWriterImpl,
};

pub struct RtpsStatelessWriterImpl<T> {
    pub writer: RtpsWriterImpl,
    pub reader_locators: Vec<RtpsReaderLocatorAttributesImpl>,
    pub heartbeat_timer: T,
    pub heartbeat_count: Count,
}

impl<T> RtpsEntityAttributes for RtpsStatelessWriterImpl<T> {
    fn guid(&self) -> Guid {
        self.writer.guid()
    }
}

impl<T> RtpsEndpointAttributes for RtpsStatelessWriterImpl<T> {
    fn topic_kind(&self) -> TopicKind {
        self.writer.endpoint.topic_kind
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.writer.endpoint.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.writer.endpoint.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.writer.endpoint.multicast_locator_list
    }
}

impl<T> RtpsWriterAttributes for RtpsStatelessWriterImpl<T> {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn push_mode(&self) -> bool {
        self.writer.push_mode
    }

    fn heartbeat_period(&self) -> Duration {
        self.writer.heartbeat_period
    }

    fn nack_response_delay(&self) -> Duration {
        self.writer.nack_response_delay
    }

    fn nack_suppression_duration(&self) -> Duration {
        self.writer.nack_suppression_duration
    }

    fn last_change_sequence_number(&self) -> SequenceNumber {
        self.writer.last_change_sequence_number
    }

    fn data_max_size_serialized(&self) -> Option<i32> {
        self.writer.data_max_size_serialized
    }

    fn writer_cache(&mut self) -> &mut Self::HistoryCacheType {
        &mut self.writer.writer_cache
    }
}

impl<'a, T> RtpsStatelessWriterAttributes<'a> for RtpsStatelessWriterImpl<T> {
    type ReaderLocatorListType = Vec<RtpsReaderLocatorOperationsImpl<'a>>;

    fn reader_locators(&'a mut self) -> Self::ReaderLocatorListType {
        let writer_cache = &self.writer.writer_cache;
        self.reader_locators
            .iter_mut()
            .map(|reader_locator_attributes| {
                RtpsReaderLocatorOperationsImpl::new(reader_locator_attributes, writer_cache)
            })
            .collect()
    }
}

impl<T: TimerConstructor> RtpsStatelessWriterConstructor for RtpsStatelessWriterImpl<T> {
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
            writer: RtpsWriterImpl::new(
                RtpsEndpointImpl::new(
                    guid,
                    topic_kind,
                    reliability_level,
                    unicast_locator_list,
                    multicast_locator_list,
                ),
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                data_max_size_serialized,
            ),
            reader_locators: Vec::new(),
            heartbeat_timer: T::new(),
            heartbeat_count: Count(0),
        }
    }
}

impl<T> RtpsStatelessWriterOperations for RtpsStatelessWriterImpl<T> {
    type ReaderLocatorType = RtpsReaderLocatorAttributesImpl;

    fn reader_locator_add(&mut self, mut a_locator: Self::ReaderLocatorType) {
        a_locator.unsent_changes = self
            .writer_cache()
            .changes()
            .iter()
            .map(|c| c.sequence_number)
            .collect();
        self.reader_locators.push(a_locator);
    }

    fn reader_locator_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::ReaderLocatorType) -> bool,
    {
        self.reader_locators.retain(|x| !f(x))
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

impl<T> RtpsWriterOperations for RtpsStatelessWriterImpl<T> {
    type DataType = Vec<u8>;
    type ParameterListType = Vec<u8>;
    type CacheChangeType = RtpsCacheChangeImpl;
    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Self::DataType,
        _inline_qos: Self::ParameterListType,
        handle: InstanceHandle,
    ) -> Self::CacheChangeType {
        self.writer.new_change(kind, data, _inline_qos, handle)
    }
}

impl<T> RtpsHistoryCacheOperations for RtpsStatelessWriterImpl<T> {
    type CacheChangeType = RtpsCacheChangeImpl;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes.push(change.sequence_number);
        }
        self.writer.writer_cache.add_change(change);
    }

    fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&Self::CacheChangeType) -> bool,
    {
        self.writer.writer_cache.remove_change(f)
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.writer.writer_cache.get_seq_num_min()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.writer.writer_cache.get_seq_num_max()
    }
}
