use rust_rtps_pim::{
    behavior::{
        reader::{
            reader::{RtpsReader, RtpsReaderOperations},
            stateful_reader::{RtpsStatefulReader, RtpsStatefulReaderOperations},
            stateless_reader::RtpsStatelessReaderOperations,
        },
        types::Duration,
    },
    structure::{
        types::{Guid, Locator, ReliabilityKind, TopicKind},
        RtpsEndpoint, RtpsEntity, RtpsHistoryCache,
    },
};

use super::{
    rtps_reader_history_cache_impl::HistoryCache, rtps_writer_proxy_impl::RtpsWriterProxyImpl,
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
    reader_cache: HistoryCache,
}

impl RtpsEntity for RtpsReaderImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}

impl RtpsReader for RtpsReaderImpl {
    type HistoryCacheType = HistoryCache;

    fn heartbeat_response_delay(&self) -> &Duration {
        &self.heartbeat_response_delay
    }

    fn heartbeat_supression_duration(&self) -> &Duration {
        &self.heartbeat_supression_duration
    }

    fn reader_cache(&self) -> &Self::HistoryCacheType {
        &self.reader_cache
    }

    fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType {
        &mut self.reader_cache
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
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
            reader_cache: HistoryCache::new(),
        }
    }
}

impl RtpsEndpoint for RtpsReaderImpl {
    fn topic_kind(&self) -> &TopicKind {
        &self.topic_kind
    }

    fn reliability_level(&self) -> &ReliabilityKind {
        &self.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
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

impl RtpsStatefulReaderOperations for RtpsReaderImpl {
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

    fn matched_writer_add(&mut self, _a_writer_proxy: <Self as RtpsStatefulReader>::WriterProxyType)
    where
        Self: RtpsStatefulReader,
    {
        todo!()
    }

    fn matched_writer_remove(&mut self, _writer_proxy_guid: &Guid) {
        todo!()
    }

    fn matched_writer_lookup(
        &self,
        _a_writer_guid: &Guid,
    ) -> Option<&<Self as RtpsStatefulReader>::WriterProxyType>
    where
        Self: RtpsStatefulReader,
    {
        todo!()
    }
}
