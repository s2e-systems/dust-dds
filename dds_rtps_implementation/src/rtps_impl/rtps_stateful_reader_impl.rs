use rust_rtps_pim::{
    behavior::{
        reader::{
            stateful_reader::{RtpsStatefulReaderConstructor, RtpsStatefulReaderOperations},
            writer_proxy::{RtpsWriterProxy, RtpsWriterProxyAttributes},
        },
        types::Duration,
    },
    messages::submessage_elements::Parameter,
    structure::{
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheGetChange},
        types::{Guid, Locator, ReliabilityKind, TopicKind},
    },
};

use super::{
    rtps_reader_history_cache_impl::{ReaderHistoryCache, ReaderHistoryCacheGetChange},
    rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};

pub struct RtpsStatefulReaderImpl<T> {
    pub guid: Guid,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub heartbeat_response_delay: Duration,
    pub heartbeat_supression_duration: Duration,
    pub reader_cache: ReaderHistoryCache<T>,
    pub expects_inline_qos: bool,
    pub matched_writers: Vec<RtpsWriterProxyImpl>,
}

impl<T> RtpsStatefulReaderConstructor for RtpsStatefulReaderImpl<T> {
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

impl<T> RtpsStatefulReaderOperations<Vec<Locator>> for RtpsStatefulReaderImpl<T> {
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

impl<'a, T> ReaderHistoryCacheGetChange<'a, T> for RtpsStatefulReaderImpl<T> {
    fn get_reader_history_cache_get_change(
        &'a self,
    ) -> &dyn RtpsHistoryCacheGetChange<&'a [Parameter<&'a [u8]>], &'a T> {
        &self.reader_cache
    }
}
