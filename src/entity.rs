use std::time::Duration;
use std::marker::PhantomData;

use crate::types::{GUID, TopicKind, ReliabilityKind, EntityId};
use crate::cache::HistoryCache;
use crate::parser::{RtpsMessage,SubMessageType,InfoTs};
use crate::Udpv4Locator;

trait Entity {
    fn get_guid(&self) -> &GUID;
}

struct RTPSEndpoint{
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
    pub unicast_locator_list: Vec<Udpv4Locator>, 
    pub multicast_locator_list: Vec<Udpv4Locator>,
    pub endpoint_id: EntityId,
}

pub trait RTPSSerializer {
    fn serialize_data() -> Vec<u8>;
}

pub trait RTPSDeserializer {
    fn deserialize_data(serialized_data: Vec<u8>) -> Self;
}

pub struct RTPSReader<D> where
D: RTPSSerializer {
    rtps_endpoint: Vec<RTPSEndpoint>,
    guid: GUID,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    expects_inline_qos: bool,
    heartbeat_response_delay: Duration,
    heartbeat_suppresion_duration: Duration,
    reader_cache: HistoryCache,
    phantom: PhantomData<D>
}

impl<D> Entity for RTPSReader<D> where
D: RTPSSerializer{
    fn get_guid(&self) -> &GUID {
        &self.guid
    }
}

impl<D> RTPSReader<D> where
D: RTPSSerializer{

    pub fn new(guid: GUID, reliability_level: ReliabilityKind, topic_kind: TopicKind, expects_inline_qos: bool) -> RTPSReader<D> {
        RTPSReader{
            guid,
            reliability_level,
            topic_kind,
            rtps_endpoint: Vec::new(),
            expects_inline_qos,
            heartbeat_response_delay: Duration::new(0,0),
            heartbeat_suppresion_duration: Duration::new(0,0),
            reader_cache: HistoryCache::new(),
            phantom: PhantomData,
        }
    }

    fn add_endpoint(&mut self, endpoint: RTPSEndpoint) {
        self.rtps_endpoint.push(endpoint);
    }

    fn get_data(&self) -> D {
        unimplemented!()
    }
}