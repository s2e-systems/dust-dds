use std::time::Duration;
use std::marker::PhantomData;

use crate::types::{GUID, TopicKind, ReliabilityKind, EntityId};
use crate::cache::HistoryCache;
use crate::parser::{RtpsMessage,SubMessageType,InfoTs};
use crate::Udpv4Locator;

trait Entity {
    fn get_guid(&self) -> &GUID;
}

pub trait RTPSEndpoint {
    fn read_data(&mut self) -> Result<Option<RtpsMessage>, ()>;
}

// struct RTPSEndpoint{
//     pub topic_kind: TopicKind,
//     pub reliability_level: ReliabilityKind,
//     pub unicast_locator_list: Vec<Udpv4Locator>, 
//     pub multicast_locator_list: Vec<Udpv4Locator>,
//     pub endpoint_id: EntityId,
// }

pub trait RTPSSerializer {
    fn serialize_data() -> Vec<u8>;
}

pub trait RTPSDeserializer {
    fn deserialize_data(serialized_data: Vec<u8>) -> Self;
}

pub struct RTPSReader<D, C> where
D: RTPSDeserializer,
C: RTPSEndpoint {
    rtps_endpoint: Vec<C>,
    guid: GUID,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    expects_inline_qos: bool,
    heartbeat_response_delay: Duration,
    heartbeat_suppresion_duration: Duration,
    reader_cache: HistoryCache,
    phantom: PhantomData<D>
}

impl<D, C> Entity for RTPSReader<D, C> where
D: RTPSDeserializer,
C: RTPSEndpoint{
    fn get_guid(&self) -> &GUID {
        &self.guid
    }
}

impl<D,C> RTPSReader<D,C> where
D: RTPSDeserializer,
C: RTPSEndpoint{

    pub fn new(
            guid: GUID,
            reliability_level: ReliabilityKind,
            topic_kind: TopicKind,
            expects_inline_qos: bool) -> RTPSReader<D,C>
    {
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

    fn add_endpoint(&mut self, endpoint: C) {
        self.rtps_endpoint.push(endpoint);
    }

    fn read_data(&mut self) {
        for endpoint in self.rtps_endpoint.iter_mut() {
            let data = endpoint.read_data().unwrap();
            if let Some(rtps_message) = data {
                self.reader_cache.process_message(rtps_message);
            }
        }
    }

    fn get_data(&self) -> D {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::collections::VecDeque;
    use crate::parser::{Data, InlineQosParameter, Payload};
    use crate::types::{ProtocolVersion, Time};

    struct MockEndpoint {
        message_buffer: VecDeque<RtpsMessage>,
    }

    impl MockEndpoint {
        fn new() -> MockEndpoint {
            MockEndpoint {
                message_buffer: VecDeque::new(),
            }
        }

        fn add_message(&mut self, message: RtpsMessage) {
            self.message_buffer.push_back(message);
        }
    }

    impl RTPSEndpoint for MockEndpoint {
        fn read_data(&mut self) -> Result<Option<RtpsMessage>,()> {
            println!("Reading data from mock endpoint");
            Ok(self.message_buffer.pop_front())
        }
    }

    struct SimpleType {
        simple: u8,
    }

    impl RTPSDeserializer for SimpleType {
        fn deserialize_data(serialized_data: Vec<u8>) -> Self {
            SimpleType {
                simple: serialized_data[0],
            }
        } 
    }

    #[test]
    fn test_reader() {
        let mut mock_endpoint = MockEndpoint::new();
        let mut rtps_message = RtpsMessage::new([0,1,2,3,4,5,6,7,8,9,10,11], [99,99], ProtocolVersion{major:2,minor:4});

        let time_submessage = SubMessageType::InfoTsSubmessage(InfoTs::new(Some(Time{seconds:10, fraction:1}))); 
        rtps_message.add_submessage(time_submessage);

        let data_submessage = SubMessageType::DataSubmessage(Data::new(
            EntityId::new([0,0,0],0),
            EntityId::new([0,1,0],1),
            1,
            Some(vec!(InlineQosParameter::KeyHash([0,1,2,3,4,5,6,7,8,9,10,11,0,1,0,1]))),
            Payload::Data(vec!(1)),
        ));
        rtps_message.add_submessage(data_submessage);

        mock_endpoint.add_message(rtps_message);

        let guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], EntityId::new([0,1,0], 0));

        let mut reader = RTPSReader::<SimpleType, MockEndpoint>::new(guid, ReliabilityKind::BestEffort, TopicKind::WithKey, false);
        reader.add_endpoint(mock_endpoint);

        reader.read_data();
        
    }
}