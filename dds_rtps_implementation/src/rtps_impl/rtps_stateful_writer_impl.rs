use rust_rtps_pim::{
    behavior::{
        stateful_writer_behavior::{
            BestEffortStatefulWriterBehavior, ReliableStatefulWriterBehavior,
            StatefulWriterBehavior,
        },
        types::Duration,
        writer::{
            reader_proxy::RtpsReaderProxyAttributes,
            stateful_writer::{
                RtpsStatefulWriterAttributes, RtpsStatefulWriterConstructor,
                RtpsStatefulWriterOperations,
            },
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    messages::types::Count,
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
};

use crate::utils::clock::{StdTimer, Timer};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl,
    rtps_history_cache_impl::{RtpsCacheChangeImpl, RtpsHistoryCacheImpl},
    rtps_reader_proxy_impl::{RtpsReaderProxyAttributesImpl, RtpsReaderProxyOperationsImpl},
    rtps_writer_impl::RtpsWriterImpl,
};

pub struct RtpsStatefulWriterImpl {
    pub writer: RtpsWriterImpl,
    pub matched_readers: Vec<RtpsReaderProxyAttributesImpl>,
    pub heartbeat_timer: StdTimer,
    pub heartbeat_count: Count,
}

impl RtpsEntityAttributes for RtpsStatefulWriterImpl {
    fn guid(&self) -> Guid {
        self.writer.endpoint.entity.guid
    }
}

impl RtpsEndpointAttributes for RtpsStatefulWriterImpl {
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

impl RtpsWriterAttributes for RtpsStatefulWriterImpl {
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

impl RtpsStatefulWriterAttributes for RtpsStatefulWriterImpl {
    type ReaderProxyType = RtpsReaderProxyAttributesImpl;

    fn matched_readers(&self) -> &[Self::ReaderProxyType] {
        &self.matched_readers
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
            matched_readers: Vec::new(),
            heartbeat_timer: StdTimer::new(),
            heartbeat_count: Count(0),
        }
    }
}

impl RtpsStatefulWriterOperations for RtpsStatefulWriterImpl {
    type ReaderProxyType = RtpsReaderProxyAttributesImpl;

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType) {
        self.matched_readers.push(a_reader_proxy)
    }

    fn matched_reader_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::ReaderProxyType) -> bool,
    {
        self.matched_readers.retain(|x| !f(x));
    }

    fn matched_reader_lookup(&self, a_reader_guid: Guid) -> Option<&Self::ReaderProxyType> {
        self.matched_readers
            .iter()
            .find(|&x| x.remote_reader_guid() == a_reader_guid)
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl RtpsWriterOperations for RtpsStatefulWriterImpl {
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

pub struct RtpsReaderProxyIterator<'a> {
    reader_proxy_iterator: std::slice::IterMut<'a, RtpsReaderProxyAttributesImpl>,
    writer_cache: &'a RtpsHistoryCacheImpl,
    last_change_sequence_number: SequenceNumber,
    reliability_level: ReliabilityKind,
    writer_guid: Guid,
    heartbeat_count: Count,
    after_heartbeat_period: bool,
}

impl<'a> Iterator for RtpsReaderProxyIterator<'a> {
    type Item = StatefulWriterBehavior<'a, RtpsReaderProxyOperationsImpl<'a>, RtpsHistoryCacheImpl>;

    fn next(&mut self) -> Option<Self::Item> {
        let reader_proxy_attributes = self.reader_proxy_iterator.next()?;
        let reader_proxy_operations =
            RtpsReaderProxyOperationsImpl::new(reader_proxy_attributes, self.writer_cache);
        match self.reliability_level {
            ReliabilityKind::BestEffort => Some(StatefulWriterBehavior::BestEffort(
                BestEffortStatefulWriterBehavior {
                    reader_proxy: reader_proxy_operations,
                    writer_cache: self.writer_cache,
                    last_change_sequence_number: self.last_change_sequence_number,
                },
            )),
            ReliabilityKind::Reliable => Some(StatefulWriterBehavior::Reliable(
                ReliableStatefulWriterBehavior {
                    reader_proxy: reader_proxy_operations,
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
    type Item = StatefulWriterBehavior<'a, RtpsReaderProxyOperationsImpl<'a>, RtpsHistoryCacheImpl>;
    type IntoIter = RtpsReaderProxyIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let heartbeat_period_duration = std::time::Duration::new(
            self.heartbeat_period().seconds as u64,
            self.heartbeat_period().fraction,
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
            writer_cache: self.writer.const_writer_cache(),
            last_change_sequence_number: self.writer.last_change_sequence_number(),
            reliability_level: self.writer.reliability_level(),
            writer_guid: self.writer.guid(),
            heartbeat_count: self.heartbeat_count,
            after_heartbeat_period,
        }
    }
}
