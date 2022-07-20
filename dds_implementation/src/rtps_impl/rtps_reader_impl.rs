use rtps_pim::{
    behavior::{reader::RtpsReaderAttributes, types::Duration},
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        history_cache::RtpsHistoryCacheConstructor,
        types::{Guid, Locator, ReliabilityKind, TopicKind},
    },
};

use super::{rtps_endpoint_impl::RtpsEndpointImpl, rtps_history_cache_impl::RtpsHistoryCacheImpl};

pub struct RtpsReaderImpl {
    endpoint: RtpsEndpointImpl,
    heartbeat_response_delay: Duration,
    heartbeat_suppression_duration: Duration,
    reader_cache: RtpsHistoryCacheImpl,
    expects_inline_qos: bool,
}

impl RtpsReaderImpl {
    pub fn new(
        endpoint: RtpsEndpointImpl,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            endpoint,
            heartbeat_response_delay,
            heartbeat_suppression_duration,
            reader_cache: RtpsHistoryCacheImpl::new(),
            expects_inline_qos,
        }
    }
}

impl RtpsEntityAttributes for RtpsReaderImpl {
    fn guid(&self) -> Guid {
        self.endpoint.guid()
    }
}

impl RtpsEndpointAttributes for RtpsReaderImpl {
    fn topic_kind(&self) -> TopicKind {
        self.endpoint.topic_kind()
    }

    fn reliability_level(&self) -> ReliabilityKind {
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
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn heartbeat_response_delay(&self) -> Duration {
        self.heartbeat_response_delay
    }

    fn heartbeat_suppression_duration(&self) -> Duration {
        self.heartbeat_suppression_duration
    }

    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType {
        &mut self.reader_cache
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
}
