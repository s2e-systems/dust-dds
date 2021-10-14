use rust_rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderOperations,
            stateful_reader::{RtpsStatefulReader, RtpsStatefulReaderOperations},
            stateless_reader::RtpsStatelessReaderOperations,
            writer_proxy::RtpsWriterProxy,
        },
        types::Duration,
    },
    structure::{
        types::{Guid, Locator, ReliabilityKind, TopicKind},
        RtpsHistoryCache,
    },
};

use super::{
    rtps_reader_history_cache_impl::ReaderHistoryCache, rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};

pub struct RtpsReaderImpl {
    guid: Guid,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    heartbeat_response_delay: Duration,
    heartbeat_supression_duration: Duration,
    expects_inline_qos: bool,
    reader_cache: ReaderHistoryCache,
}

impl RtpsReaderOperations for RtpsReaderImpl {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list: unicast_locator_list.into_iter().cloned().collect(),
            multicast_locator_list: multicast_locator_list.into_iter().cloned().collect(),
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
            reader_cache: ReaderHistoryCache::new(),
        }
    }
}

impl RtpsStatelessReaderOperations for RtpsReaderImpl {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        <Self as RtpsReaderOperations>::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        )
    }
}

impl RtpsStatefulReader for RtpsReaderImpl {
    type WriterProxyType = RtpsWriterProxyImpl;

    fn matched_writers(&self) -> &[Self::WriterProxyType] {
        todo!()
    }
}

impl<L> RtpsStatefulReaderOperations<L> for RtpsReaderImpl {
    fn matched_writer_add(&mut self, _a_writer_proxy: RtpsWriterProxy<L>)
    where
        Self: RtpsStatefulReader,
    {
        todo!()
    }

    fn matched_writer_remove(&mut self, _writer_proxy_guid: &Guid) {
        todo!()
    }

    fn matched_writer_lookup(&self, _a_writer_guid: &Guid) -> Option<&RtpsWriterProxy<L>>
    where
        Self: RtpsStatefulReader,
    {
        todo!()
    }
}
