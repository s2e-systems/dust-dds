use dds_transport::types::Locator;

use crate::{dcps_psm::Duration, implementation::rtps::history_cache::RtpsHistoryCacheImpl};

use super::{
    endpoint::RtpsEndpointImpl,
    reader::RtpsReaderImpl,
    types::{Guid, ReliabilityKind, TopicKind},
};

pub struct RtpsStatelessReaderImpl(RtpsReaderImpl);

impl RtpsStatelessReaderImpl {
    pub fn guid(&self) -> Guid {
        self.0.guid()
    }
}

impl RtpsStatelessReaderImpl {
    pub fn topic_kind(&self) -> TopicKind {
        self.0.topic_kind()
    }

    pub fn reliability_level(&self) -> ReliabilityKind {
        self.0.reliability_level()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.0.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.0.multicast_locator_list()
    }
}

impl RtpsStatelessReaderImpl {
    pub fn heartbeat_response_delay(&self) -> Duration {
        self.0.heartbeat_response_delay()
    }

    pub fn heartbeat_suppression_duration(&self) -> Duration {
        self.0.heartbeat_suppression_duration()
    }

    pub fn reader_cache(&mut self) -> &mut RtpsHistoryCacheImpl {
        self.0.reader_cache()
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.0.expects_inline_qos()
    }
}

impl RtpsStatelessReaderImpl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        if reliability_level == ReliabilityKind::Reliable {
            panic!("Reliable stateless reader is not supported");
        }

        Self(RtpsReaderImpl::new(
            RtpsEndpointImpl::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
            ),
            heartbeat_response_delay,
            heartbeat_suppression_duration,
            expects_inline_qos,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::types::{EntityId, GuidPrefix, USER_DEFINED_READER_NO_KEY};

    use super::*;

    #[test]
    #[should_panic]
    fn reliable_stateless_data_reader_not_constructible() {
        RtpsStatelessReaderImpl::new(
            Guid::new(
                GuidPrefix([3; 12]),
                EntityId::new([4, 1, 3], USER_DEFINED_READER_NO_KEY),
            ),
            TopicKind::NoKey,
            ReliabilityKind::Reliable,
            &[],
            &[],
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
        );
    }
}
