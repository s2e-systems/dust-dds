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

pub struct RTPSReader<'a, D> where
  D: RTPSDeserializer {
    rtps_endpoint: Vec<RTPSEndpoint>,
    guid: GUID,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    expects_inline_qos: bool,
    heartbeat_response_delay: Duration,
    heartbeat_suppresion_duration: Duration,
    reader_cache: HistoryCache,
    phantom: PhantomData<&'a D>,
}

impl<D> Entity for RTPSReader<'_, D> where
    D: RTPSDeserializer {

    fn get_guid(&self) -> &GUID {
        &self.guid
    }
}

impl<'a, D> RTPSReader<'a, D> where
    D: RTPSDeserializer {

    pub fn new(guid: GUID, reliability_level: ReliabilityKind, topic_kind: TopicKind) -> RTPSReader<'a, D> {
        RTPSReader{
            guid,
            reliability_level,
            topic_kind,
            rtps_endpoint: Vec::new(),
            expects_inline_qos: false,
            heartbeat_response_delay: Duration::new(0,0),
            heartbeat_suppresion_duration: Duration::new(0,0),
            reader_cache: HistoryCache::new(),
            phantom: PhantomData,
        }
    }

    fn add_endpoint(&mut self, endpoint: RTPSEndpoint) {
        self.rtps_endpoint.push(endpoint);
    }

    fn process_message(&mut self, message: RtpsMessage) {
        let source_guid_prefix = message.guid_prefix(); 
        let source_version = message.protocol_version();
        let source_vendor_id = message.vendor_id();

        for submessage in message.submessages() {
            if let SubMessageType::InfoTsSubmessage(info_ts) = submessage {
                println!("Got info timestamp: {:?}", info_ts.timestamp() );
            } else if let SubMessageType::DataSubmessage(data) = submessage {
                println!("Got data submessage");
            } else {
                println!("Unsupported message received");
            }
        }
    }

    pub fn get_data(&self) -> &D {
        unimplemented!()
    }
}