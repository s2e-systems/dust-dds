use rtps_pim::{
    behavior::types::Duration,
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

use super::{endpoint::RtpsEndpointImpl, history_cache::RtpsHistoryCacheImpl};

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

impl RtpsReaderImpl {
    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }
}

impl RtpsReaderImpl {
    pub fn topic_kind(&self) -> TopicKind {
        self.endpoint.topic_kind()
    }

    pub fn reliability_level(&self) -> ReliabilityKind {
        self.endpoint.reliability_level()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}

impl RtpsReaderImpl {
    pub fn heartbeat_response_delay(&self) -> Duration {
        self.heartbeat_response_delay
    }

    pub fn heartbeat_suppression_duration(&self) -> Duration {
        self.heartbeat_suppression_duration
    }

    pub fn reader_cache(&mut self) -> &mut RtpsHistoryCacheImpl {
        &mut self.reader_cache
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
}
