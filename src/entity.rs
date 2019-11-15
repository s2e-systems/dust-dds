use std::time::Duration;
use crate::types::{GUID, TopicKind, ReliabilityLevel, EntityId};
use crate::cache::Cache;
use crate::Udpv4Locator;

trait Entity {
    fn get_guid(&self) -> &GUID;
}

#[derive(Default)]
struct RTPSEntity{
    guid: GUID,
}

struct RTPSEndpoint{
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityLevel,
    pub unicast_locator_list: Vec<Udpv4Locator>, 
    pub multicast_locator_list: Vec<Udpv4Locator>,
    pub endpoint_id: EntityId,
}

impl Default for RTPSEndpoint{
    fn default() -> Self {
        RTPSEndpoint {
            topic_kind: 0,
            reliability_level: 0,
            unicast_locator_list: Vec::new(),
            multicast_locator_list: Vec::new(),
            endpoint_id: EntityId::default(),
        }
    }
}

pub struct RTPSReader<D>{
    rtps_entity: RTPSEntity,
    rtps_endpoint: RTPSEndpoint,
    expects_inline_qos: bool,
    heartbeat_response_delay: Duration,
    heartbeat_suppresion_duration: Duration,
    reader_cache: Cache<D>
}

impl<D> Entity for RTPSReader<D> {
    fn get_guid(&self) -> &GUID {
        &self.rtps_entity.guid
    }
}

impl<D> RTPSReader<D> {
    pub fn new() -> RTPSReader<D> {
        RTPSReader::<D> {
            rtps_entity: RTPSEntity::default(),
            rtps_endpoint: RTPSEndpoint::default(),
            expects_inline_qos: false,
            heartbeat_response_delay: Duration::new(0,0),
            heartbeat_suppresion_duration: Duration::new(0,0),
            reader_cache: Cache::new(),
        }
    }
}

// struct EndPoint {
//     topic_kind: TopicKind,
//     reliability_level: ReliabilityLevelT

// }