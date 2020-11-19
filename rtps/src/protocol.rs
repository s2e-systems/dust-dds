use std::sync::{Arc, Mutex};

use crate::types::{GUID, EntityId, EntityKind};
use crate::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID};
use crate::transport::Transport;
// use crate::discovery::spdp::{SimpleParticipantDiscoveryProtocol, SPDPdiscoveredParticipantData};
use crate::endpoint_types::BuiltInEndpointSet;
use crate::structure::{RtpsParticipant, RtpsGroup, RtpsEntity};
use crate::message_receiver::RtpsMessageReceiver;
use crate::message_sender::RtpsMessageSender;
use crate::subscriber::Subscriber;
use crate::publisher::Publisher;
use crate::discovery::builtin_publisher::BuiltInPublisher;
use crate::discovery::builtin_subscriber::BuiltInSubscriber;


use rust_dds_interface::types::{DomainId, InstanceHandle};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolParticipant, ProtocolSubscriber, ProtocolPublisher};

pub struct RtpsProtocol {
    participant: RtpsParticipant,
    builtin_publisher: BuiltInPublisher,
    builtin_subscriber: BuiltInSubscriber,
    userdata_transport: Box<dyn Transport>,
    metatraffic_transport: Box<dyn Transport>,
    publisher_counter: usize,
    subscriber_counter: usize,
}

impl RtpsProtocol {
    pub fn new(_domain_id: DomainId, userdata_transport: impl Transport, metatraffic_transport: impl Transport, _domain_tag: String, lease_duration: rust_dds_interface::types::Duration) -> Self {

        let guid_prefix = [1,2,3,4,5,6,7,8,9,10,11,12];  //TODO: Should be uniquely generated
        // let participant = RtpsParticipant::new(domain_id, guid_prefix, PROTOCOL_VERSION_2_4, VENDOR_ID);

        let _lease_duration = crate::behavior::types::Duration::from_secs(lease_duration.sec as u64); // TODO: Fix this conversion

        // let data = SPDPdiscoveredParticipantData::new(
        //     participant.domain_id(),
        //     domain_tag.clone(), 
        //     participant.protocol_version(), 
        //     participant.guid().prefix(), 
        //     participant.vendor_id(), 
        //     metatraffic_transport.unicast_locator_list().clone(), 
        //     metatraffic_transport.multicast_locator_list().clone(), 
        //     userdata_transport.unicast_locator_list().clone(),
        //     userdata_transport.multicast_locator_list().clone(),
        //     BuiltInEndpointSet::new(
        //         BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER | 
        //         BuiltInEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR |
        //         BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER |
        //         BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR |
        //         BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER |
        //         BuiltInEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR |
        //         BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER |
        //         BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR
        //      ),
        //     lease_duration,
        // );

        let builtin_publisher = BuiltInPublisher::new(guid_prefix);
        let builtin_subscriber = BuiltInSubscriber::new(guid_prefix);

        let userdata_transport = Box::new(userdata_transport);
        let metatraffic_transport = Box::new(metatraffic_transport);

        // Self {
        //     participant,
        //     builtin_publisher,
        //     builtin_subscriber,
        //     userdata_transport,
        //     metatraffic_transport,
        //     publisher_counter: 0,
        //     subscriber_counter: 0,
        // }
        todo!()
    }

    pub fn receive_metatraffic(&self) {
        // RtpsMessageReceiver::receive(
        //     self.participant.guid().prefix(), 
        //     self.metatraffic_transport.as_ref(),
        //     self.builtin_publisher.lock().unwrap().iter()
        //     .chain(self.builtin_subscriber.lock().unwrap().iter()))
    }

    pub fn send_metatraffic(&self) {
        // RtpsMessageSender::send(
        //     self.participant.guid().prefix(), 
        //     self.metatraffic_transport.as_ref(),
        //     self.builtin_publisher.lock().unwrap().iter()
        //     .chain(self.builtin_subscriber.lock().unwrap().iter()))
    }
}

impl ProtocolEntity for RtpsProtocol {
    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
        // self.participant.guid().into()
    }
}

impl ProtocolParticipant for RtpsProtocol {
    fn create_publisher(&mut self) ->  Box<dyn ProtocolPublisher> {
        // let guid_prefix = self.participant.guid().prefix();
        // let entity_id = EntityId::new([self.publisher_counter as u8,0,0], EntityKind::UserDefinedWriterGroup);
        // self.publisher_counter += 1;
        // let publisher_guid = GUID::new(guid_prefix, entity_id);
        // let publisher_group = RtpsGroup::new(publisher_guid);
        // // self.user_defined_groups.push(publisher_group.clone());

        // Box::new(Publisher::new(publisher_group))
        todo!()
    }

    fn create_subscriber(&mut self) -> Box<dyn ProtocolSubscriber> {
        // let guid_prefix = self.participant.guid().prefix();
        // let entity_id = EntityId::new([self.subscriber_counter as u8,0,0], EntityKind::UserDefinedReaderGroup);
        // self.subscriber_counter += 1;
        // let subscriber_guid = GUID::new(guid_prefix, entity_id);
        // let subscriber_group = RtpsGroup::new(subscriber_guid);

        // Box::new(Subscriber::new(subscriber_group))
        todo!()
    }

    fn get_builtin_subscriber(&self) -> Box<dyn ProtocolSubscriber> {
        todo!()
        // Box::new(Subscriber::new(self.builtin_subscriber.clone()))
    }

    fn run(&self) {
        // RtpsMessageReceiver::receive(
        //     self.participant.guid().prefix(), 
        //     self.metatraffic_transport.as_ref(),
        //     self.builtin_publisher.lock().unwrap().iter()
        //     .chain(self.builtin_subscriber.lock().unwrap().iter()));

        // RtpsMessageSender::send(
        //         self.participant.guid().prefix(), 
        //         self.metatraffic_transport.as_ref(),
        //         self.builtin_publisher.lock().unwrap().iter()
        //         .chain(self.builtin_subscriber.lock().unwrap().iter()));
    }

    fn receive(&self, _publisher_list: &[&dyn ProtocolPublisher], _subscriber_list: &[&dyn ProtocolSubscriber]) {
        todo!()
    }

    fn send(&self, _publisher_list: &[&dyn ProtocolPublisher], _subscriber_list: &[&dyn ProtocolSubscriber]) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use std::cell::RefCell;
    // use crate::behavior_types::Duration;
    // use crate::types::{GUID, Locator};
    // use crate::behavior::{StatelessReader, StatefulReader, StatefulWriter};
    // use crate::messages::{Endianness, RtpsMessage, RtpsSubmessage};
    // use crate::messages::submessages::{Data, data_submessage::Payload};
    // use crate::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID};
    // use crate::types::constants::{
    //     ENTITYID_UNKNOWN,
    //     ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
    //     ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR,
    //     ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    //     ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    //     ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    //     ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    //     ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    //     ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR};
    // use crate::inline_qos_types::{StatusInfo, KeyHash};

    // struct MockTransport{
    //     multicast_locator_list: Vec<Locator>,
    //     unicast_locator_list: Vec<Locator>,
    // }

    // impl MockTransport{
    //     fn new() -> Self {
    //         Self {
    //             multicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
    //             unicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
    //         }
    //     }
    // }

    // impl Transport for MockTransport {
    //     fn write(&self, message: crate::RtpsMessage, _destination_locator: &Locator) {
    //         println!("{:?}", message);
    //     }

    //     fn read(&self) -> crate::transport::TransportResult<Option<(crate::RtpsMessage, Locator)>> {
    //         todo!()
    //     }

    //     fn unicast_locator_list(&self) -> &Vec<Locator> {
    //         &self.unicast_locator_list
    //     }

    //     fn multicast_locator_list(&self) -> &Vec<Locator> {
    //         &self.multicast_locator_list
    //     }
    // }


    // #[test]
    // fn create_delete_publisher() {
    //     let domain_id = 1;
    //     let domain_tag = "".to_string();
    //     let lease_duration = rust_dds_interface::types::Duration{sec: 30, nanosec: 0};
    //     let mut protocol = RtpsProtocol::new(domain_id, MockTransport::new(), MockTransport::new(), domain_tag, lease_duration);

    //     assert_eq!(protocol.participant.mut_groups().len(), 0);
    //     let publisher1 = protocol.create_publisher();
    //     assert_eq!(protocol.participant.mut_groups().len(), 1);
    //     let _publisher2 = protocol.create_publisher();
    //     assert_eq!(protocol.participant.mut_groups().len(), 2);
        
    //     protocol.delete_publisher(&publisher1);
    //     assert_eq!(protocol.participant.mut_groups().len(), 1);
    // }

    // #[test]
    // fn create_delete_subscriber() {
    //     let domain_id = 1;
    //     let domain_tag = "".to_string();
    //     let lease_duration = rust_dds_interface::types::Duration{sec: 30, nanosec: 0};
    //     let mut protocol = RtpsProtocol::new(domain_id, MockTransport::new(), MockTransport::new(), domain_tag, lease_duration);

    //     assert_eq!(protocol.participant.mut_groups().len(), 0);
    //     let subscriber1 = protocol.create_subscriber();
    //     assert_eq!(protocol.participant.mut_groups().len(), 1);
    //     let _subscriber2 = protocol.create_subscriber();
    //     assert_eq!(protocol.participant.mut_groups().len(), 2);
        
    //     protocol.delete_subscriber(&subscriber1);
    //     assert_eq!(protocol.participant.mut_groups().len(), 1);
    // }

    
    // #[test]
    // fn spdp_announce() {
    //     let domain_id = 0;
    //     let domain_tag = "".to_string();
    //     let lease_duration = rust_dds_interface::types::Duration{sec: 30, nanosec: 0};
    //     let protocol = RtpsProtocol::new(domain_id, MockTransport::new(), MockTransport::new(), domain_tag, lease_duration);
    //     protocol.send_metatraffic();
    // }


    // struct MockTransportDetect{
    //     multicast_locator_list: Vec<Locator>,
    //     unicast_locator_list: Vec<Locator>,
    //     to_read: RefCell<Vec<(crate::RtpsMessage, Locator)>>
    // }

    // impl MockTransportDetect{
    //     fn new() -> Self {            
    //         Self {
    //             multicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
    //             unicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
    //             to_read: RefCell::new(vec![])
    //         }
    //     }
    // }

    // impl Transport for MockTransportDetect {
    //     fn write(&self, message: crate::RtpsMessage, _destination_locator: &Locator) {
    //         println!("{:?}", message);
    //     }

    //     fn read(&self) -> crate::transport::TransportResult<Option<(crate::RtpsMessage, Locator)>> {
    //         Ok(self.to_read.borrow_mut().pop())
    //     }

    //     fn unicast_locator_list(&self) -> &Vec<Locator> {
    //         &self.unicast_locator_list
    //     }

    //     fn multicast_locator_list(&self) -> &Vec<Locator> {
    //         &self.multicast_locator_list
    //     }
    // }


  
    // #[test]
    // fn spdp_detect() { 
    //     let domain_id = 0;
    //     let domain_tag = "".to_string();
    //     let lease_duration = rust_dds_interface::types::Duration{sec: 30, nanosec: 0};
    //     let transport = MockTransportDetect::new();

    //     let locator = Locator::new_udpv4(7401, [127,0,0,1]);
    //     let participant_guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8 ,9, 10, 11, 12];

    //     let remote_participant_guid_prefix = [2; 12];
    //     let unicast_locator_list = vec![Locator::new_udpv4(7401, [127,0,0,1])];
    //     let multicast_locator_list = vec![Locator::new_udpv4(7401, [127,0,0,1])];
    //     let expected = SPDPdiscoveredParticipantData::new(
    //         0,
    //         "".to_string(), 
    //         PROTOCOL_VERSION_2_4, 
    //         remote_participant_guid_prefix, 
    //         VENDOR_ID, 
    //         unicast_locator_list.clone(), 
    //         multicast_locator_list.clone(), 
    //         unicast_locator_list.clone(),
    //         multicast_locator_list.clone(),
    //         BuiltInEndpointSet::new(
    //             BuiltInEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER | 
    //             BuiltInEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR),
    //         Duration::from_millis(100),
    //     );
    //     let mut parameter_list = ParameterList{parameter:Vec::new()};
    //     parameter_list.push(StatusInfo([0,0,0,0]));
    //     parameter_list.push(KeyHash(expected.key()));
    //     let inline_qos = Some(parameter_list);
    //     let data_submessage = Data::new(Endianness::LittleEndian, ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 0, inline_qos, Payload::Data(expected.data(CdrEndianness::LittleEndian)));
    //     let message = RtpsMessage::new(
    //         PROTOCOL_VERSION_2_4,
    //         VENDOR_ID,
    //         participant_guid_prefix, vec![RtpsSubmessage::Data(data_submessage)]);


    //     transport.to_read.borrow_mut().push((message, locator));

    //     let protocol = RtpsProtocol::new(domain_id, MockTransportDetect::new(), transport, domain_tag, lease_duration);
    //     protocol.receive_metatraffic();

    //     let builtin_subscriber = protocol.builtin_subscriber.lock().unwrap();
    //     let builtin_publisher = protocol.builtin_publisher.lock().unwrap();
    //     {
    //         let mut first_endpoint = builtin_subscriber.endpoints().into_iter().next().unwrap().lock().unwrap();

    //         assert!(first_endpoint.guid().entity_id() == ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR);       
                
    //         let spdp_detector = first_endpoint.get_mut::<StatelessReader>().unwrap();

    //         let cache = spdp_detector.reader_cache();
    //         let cc = cache.changes().iter().next().unwrap();        
    //         let result = SPDPdiscoveredParticipantData::from_key_data( cc.instance_handle(), cc.data_value(), 0);
    //         assert!(result == expected);
    //     }

    //     {
    //         let sedp_builtin_publications_detector = builtin_subscriber.endpoints().iter().find(|&x| x.lock().unwrap().guid().entity_id() == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR).unwrap().lock().unwrap();
    //         let sedp_builtin_publications_detector = sedp_builtin_publications_detector.get::<StatefulReader>().unwrap();
    //         assert!(sedp_builtin_publications_detector.matched_writer_lookup(GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)).is_some());
    //     }

    //     {
    //         let sedp_builtin_topics_announcer = builtin_publisher.endpoints().iter().find(|&x| x.lock().unwrap().guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER).unwrap().lock().unwrap();
    //         let sedp_builtin_topics_announcer = sedp_builtin_topics_announcer.get::<StatefulWriter>().unwrap();
    //         assert!(sedp_builtin_topics_announcer.matched_reader_lookup(GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR)).is_some());
    //     }

    //     {
    //         let sedp_builtin_subscriptions_detector = builtin_subscriber.endpoints().iter().find(|&x| x.lock().unwrap().guid().entity_id() == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR).unwrap().lock().unwrap();
    //         let sedp_builtin_subscriptions_detector = sedp_builtin_subscriptions_detector.get::<StatefulReader>().unwrap();
    //         assert!(sedp_builtin_subscriptions_detector.matched_writer_lookup(GUID::new(remote_participant_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER)).is_none());
    //     }

    // }
}