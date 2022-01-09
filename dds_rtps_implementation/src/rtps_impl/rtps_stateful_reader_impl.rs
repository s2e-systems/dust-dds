use rust_rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{RtpsStatefulReaderConstructor, RtpsStatefulReaderOperations},
            writer_proxy::{RtpsWriterProxy, RtpsWriterProxyAttributes},
        },
        types::Duration,
    },
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        history_cache::RtpsHistoryCacheConstructor,
        types::{Guid, Locator, ReliabilityKind, TopicKind},
    },
};

use super::{
    rtps_reader_history_cache_impl::ReaderHistoryCache, rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};

pub struct RtpsStatefulReaderImpl {
    guid: Guid,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    heartbeat_response_delay: Duration,
    heartbeat_supression_duration: Duration,
    reader_cache: ReaderHistoryCache,
    expects_inline_qos: bool,
    matched_writers: Vec<RtpsWriterProxyImpl>,
}

impl RtpsStatefulReaderConstructor for RtpsStatefulReaderImpl {
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
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            heartbeat_response_delay,
            heartbeat_supression_duration,
            reader_cache: ReaderHistoryCache::new(),
            expects_inline_qos,
            matched_writers: Vec::new(),
        }
    }
}

impl RtpsStatefulReaderOperations<Vec<Locator>> for RtpsStatefulReaderImpl {
    type WriterProxyType = RtpsWriterProxyImpl;

    fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy<Vec<Locator>>) {
        let writer_proxy = RtpsWriterProxyImpl::new(a_writer_proxy);
        self.matched_writers.push(writer_proxy);
    }

    fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != writer_proxy_guid)
    }

    fn matched_writer_lookup(&self, a_writer_guid: &Guid) -> Option<&Self::WriterProxyType> {
        self.matched_writers
            .iter()
            .find(|&x| x.remote_writer_guid() == a_writer_guid)
    }
}

impl RtpsReaderAttributes for RtpsStatefulReaderImpl {
    type ReaderHistoryCacheType = ReaderHistoryCache;

    fn heartbeat_response_delay(&self) -> &Duration {
        &self.heartbeat_response_delay
    }

    fn heartbeat_supression_duration(&self) -> &Duration {
        &self.heartbeat_supression_duration
    }

    fn reader_cache(&self) -> &Self::ReaderHistoryCacheType {
        &self.reader_cache
    }

    fn expects_inline_qos(&self) -> &bool {
        &self.expects_inline_qos
    }
}

impl RtpsEndpointAttributes for RtpsStatefulReaderImpl {
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

impl RtpsEntityAttributes for RtpsStatefulReaderImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}
