use rust_rtps_pim::{
    behavior::{
        types::Duration,
        reader::reader::RtpsReaderAttributes,
        stateless_reader_behavior::BestEffortStatelessReaderBehavior
    },
    structure::{
        entity::RtpsEntityAttributes,
        types::{Guid, TopicKind, ReliabilityKind, Locator},
        endpoint::RtpsEndpointAttributes,
        history_cache::RtpsHistoryCacheConstructor
    },
};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl,
    rtps_reader_history_cache_impl::ReaderHistoryCache,
};

pub struct RtpsReaderImpl {
    endpoint: RtpsEndpointImpl,
    heartbeat_response_delay: Duration,
    heartbeat_supression_duration: Duration,
    reader_cache: ReaderHistoryCache,
    expects_inline_qos: bool,
}

impl RtpsReaderImpl {
    pub fn new(
        endpoint: RtpsEndpointImpl,
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            endpoint,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            reader_cache: ReaderHistoryCache::new(),
            expects_inline_qos,
        }
    }
}

impl RtpsEntityAttributes for RtpsReaderImpl {
    fn guid(&self) -> &Guid {
        self.endpoint.guid()
    }
}

impl RtpsEndpointAttributes for RtpsReaderImpl {
    fn topic_kind(&self) -> &TopicKind {
        self.endpoint.topic_kind()
    }

    fn reliability_level(&self) -> &ReliabilityKind {
        self.endpoint.reliability_level()
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}

impl RtpsReaderAttributes for RtpsReaderImpl {
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

impl<'a> From<&'a mut RtpsReaderImpl>
for BestEffortStatelessReaderBehavior<'a, ReaderHistoryCache>
{
    fn from(reader: &'a mut RtpsReaderImpl) -> Self {
        BestEffortStatelessReaderBehavior {
            reader_guid: reader.endpoint.guid(),
            reader_cache: &mut reader.reader_cache,
        }
    }
}