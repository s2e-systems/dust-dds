use rust_rtps_pim::{
    behavior::{
        reader::{
            reader::{RtpsReader, RTPSReaderOperations},
            stateful_reader::{RTPSStatefulReader, RTPSStatefulReaderOperations},
            stateless_reader::RTPSStatelessReaderOperations,
        },
        types::Duration,
    },
    structure::{
        types::{Locator, ReliabilityKind, TopicKind, GUID},
        RTPSEndpoint, RtpsEntity, RtpsHistoryCache,
    },
};

use super::{
    rtps_history_cache_impl::RtpsHistoryCacheImpl, rtps_writer_proxy_impl::RTPSWriterProxyImpl,
};

pub struct RtpsReaderImpl {
    guid: GUID,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    heartbeat_response_delay: Duration,
    heartbeat_supression_duration: Duration,
    expects_inline_qos: bool,
    reader_cache: RtpsHistoryCacheImpl,
}

impl RtpsEntity for RtpsReaderImpl {
    fn guid(&self) -> &GUID {
        &self.guid
    }
}

impl RtpsReader for RtpsReaderImpl {
    type HistoryCacheType = RtpsHistoryCacheImpl;

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

impl RTPSReaderOperations for RtpsReaderImpl {
    fn new(
        guid: GUID,
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
            reader_cache: RtpsHistoryCacheImpl::new(),
        }
    }
}

impl RTPSEndpoint for RtpsReaderImpl {
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

impl RTPSStatelessReaderOperations for RtpsReaderImpl {
    fn new(
        guid: GUID,
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
            reader_cache: RtpsHistoryCacheImpl::new(),
        }
    }
}

impl RTPSStatefulReader for RtpsReaderImpl {
    type WriterProxyType = RTPSWriterProxyImpl;

    fn matched_writers(&self) -> &[Self::WriterProxyType] {
        todo!()
    }
}

impl RTPSStatefulReaderOperations for RtpsReaderImpl {
    fn new(
        _guid: GUID,
        _topic_kind: TopicKind,
        _reliability_level: ReliabilityKind,
        _unicast_locator_list: &[Locator],
        _multicast_locator_list: &[Locator],
        _heartbeat_response_delay: Duration,
        _heartbeat_supression_duration: Duration,
        _expects_inline_qos: bool,
    ) -> Self {
        todo!()
    }

    fn matched_writer_add(&mut self, _a_writer_proxy: <Self as RTPSStatefulReader>::WriterProxyType)
    where
        Self: RTPSStatefulReader,
    {
        todo!()
    }

    fn matched_writer_remove(&mut self, _writer_proxy_guid: &GUID) {
        todo!()
    }

    fn matched_writer_lookup(
        &self,
        _a_writer_guid: &GUID,
    ) -> Option<&<Self as RTPSStatefulReader>::WriterProxyType>
    where
        Self: RTPSStatefulReader,
    {
        todo!()
    }
}
