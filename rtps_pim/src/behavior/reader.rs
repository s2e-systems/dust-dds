use crate::{
    behavior,
    structure::{self, RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
};

pub struct RTPSReader<
    PSM: structure::Types + behavior::Types,
    HistoryCache: RTPSHistoryCache<PSM>,
> {
    pub endpoint: RTPSEndpoint<PSM>,
    pub expects_inline_qos: bool,
    pub heartbeat_response_delay: PSM::Duration,
    pub heartbeat_supression_duration: PSM::Duration,
    pub reader_cache: HistoryCache,
}

impl<PSM: structure::Types + behavior::Types, HistoryCache: RTPSHistoryCache<PSM>>
    RTPSReader<PSM, HistoryCache>
{
    pub fn new(
        guid: PSM::Guid,
        topic_kind: PSM::TopicKind,
        reliability_level: PSM::ReliabilityKind,
        unicast_locator_list: PSM::LocatorVector,
        multicast_locator_list: PSM::LocatorVector,
        expects_inline_qos: bool,
        heartbeat_response_delay: PSM::Duration,
        heartbeat_supression_duration: PSM::Duration,
    ) -> Self {
        let entity = RTPSEntity::new(guid);
        let endpoint = RTPSEndpoint {
            entity,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
        };

        Self {
            endpoint,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            reader_cache: HistoryCache::new(),
        }
    }
}
