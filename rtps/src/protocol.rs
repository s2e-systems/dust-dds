use std::sync::Arc;
use crate::transport::Transport;
use crate::behavior::types::Duration;
use crate::discovery::spdp::{SimpleParticipantDiscoveryProtocol, SPDPdiscoveredParticipantData};
use crate::discovery::sedp::{SimpleEndpointDiscoveryProtocol};
use crate::endpoint_types::BuiltInEndpointSet;
use crate::structure::RtpsParticipant;
use crate::structure::entity::RtpsEntity;
use crate::message_receiver::RtpsMessageReceiver;
use crate::message_sender::RtpsMessageSender;

use rust_dds_interface::types::DomainId;

pub struct RtpsProtocol {
    participant: RtpsParticipant,
    spdp: SimpleParticipantDiscoveryProtocol,
    userdata_transport: Arc<dyn Transport>,
    metatraffic_transport: Arc<dyn Transport>,
}

impl RtpsProtocol {
    pub fn new(domain_id: DomainId, userdata_transport: impl Transport, metatraffic_transport: impl Transport, domain_tag: String, lease_duration: Duration) -> Self {

        let guid_prefix = [1,2,3,4,5,6,7,8,9,10,11,12];  //TODO: Should be uniquely generated
        let participant = RtpsParticipant::new(domain_id, guid_prefix);

        let data = SPDPdiscoveredParticipantData::new(
            participant.domain_id(),
            domain_tag, 
            participant.protocol_version(), 
            participant.guid().prefix(), 
            participant.vendor_id(), 
            metatraffic_transport.unicast_locator_list().clone(), 
            metatraffic_transport.multicast_locator_list().clone(), 
            userdata_transport.unicast_locator_list().clone(),
            userdata_transport.multicast_locator_list().clone(),
            BuiltInEndpointSet::new(0),
            lease_duration,
        );
        let sedp = SimpleEndpointDiscoveryProtocol::new(guid_prefix);
        let spdp = SimpleParticipantDiscoveryProtocol::new(data, &sedp);

        {
            let mut builtin_publisher = participant.builtin_publisher().lock().unwrap();
            let mut builtin_subscriber = participant.builtin_subscriber().lock().unwrap();
            let builtin_publisher_endpoints = builtin_publisher.mut_endpoints();
            let builtin_subscriber_endpoints = builtin_subscriber.mut_endpoints();
            builtin_publisher_endpoints.push(spdp.spdp_builtin_participant_writer().clone());
            builtin_subscriber_endpoints.push(spdp.spdp_builtin_participant_reader().clone());

            //SEDP
            
            builtin_publisher_endpoints.push(sedp.sedp_builtin_publications_writer().clone());
            builtin_publisher_endpoints.push(sedp.sedp_builtin_subscriptions_writer().clone());
            builtin_publisher_endpoints.push(sedp.sedp_builtin_topics_writer().clone());
            builtin_subscriber_endpoints.push(sedp.sedp_builtin_publications_reader().clone());
            builtin_subscriber_endpoints.push(sedp.sedp_builtin_subscriptions_reader().clone());
            builtin_subscriber_endpoints.push(sedp.sedp_builtin_topics_reader().clone());
        }
        let userdata_transport = Arc::new(userdata_transport);
        let metatraffic_transport = Arc::new(metatraffic_transport);

        Self {
            participant,
            spdp,
            userdata_transport,
            metatraffic_transport,
        }
    }

    pub fn run_builtin_endpoints(&self) {
        // self.spdp.spdp_builtin_participant_reader().lock().unwrap().run(
        //     |_| println!("Discovery data")
        // );

        self.spdp.spdp_builtin_participant_writer().lock().unwrap().run();
    }
    pub fn receive_metatraffic(&self) {
        RtpsMessageReceiver::receive(
            self.participant.guid().prefix(), 
            self.metatraffic_transport.as_ref(),
            self.participant.builtin_publisher().lock().unwrap().into_iter()
            .chain(self.participant.builtin_subscriber().lock().unwrap().into_iter()))
    }

    pub fn send_metatraffic(&self) {
        RtpsMessageSender::send(
            self.participant.guid().prefix(), 
            self.metatraffic_transport.as_ref(),
            self.participant.builtin_publisher().lock().unwrap().into_iter()
            .chain(self.participant.builtin_subscriber().lock().unwrap().into_iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use crate::behavior_types::Duration;
    use crate::types::{Locator};
    use crate::behavior::StatelessReader;
    use crate::messages::{Endianness, RtpsMessage, RtpsSubmessage};
    use crate::messages::submessages::{Data, data_submessage::Payload};
    use crate::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID};
    use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR, ENTITYID_UNKNOWN};
    use crate::serialized_payload::{ParameterList, CdrEndianness};
    use crate::inline_qos_types::{StatusInfo, KeyHash};

    struct MockTransport{
        multicast_locator_list: Vec<Locator>,
        unicast_locator_list: Vec<Locator>,
    }

    impl MockTransport{
        fn new() -> Self {
            Self {
                multicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
                unicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
            }
        }
    }

    impl Transport for MockTransport {
        fn write(&self, message: crate::RtpsMessage, _destination_locator: &Locator) {
            println!("{:?}", message);
        }

        fn read(&self) -> crate::transport::TransportResult<Option<(crate::RtpsMessage, Locator)>> {
            todo!()
        }

        fn unicast_locator_list(&self) -> &Vec<Locator> {
            &self.unicast_locator_list
        }

        fn multicast_locator_list(&self) -> &Vec<Locator> {
            &self.multicast_locator_list
        }
    }

    #[test]
    fn spdp_announce() {
        let domain_id = 0;
        let domain_tag = "".to_string();
        let lease_duration = Duration::from_millis(100);
        let protocol = RtpsProtocol::new(domain_id, MockTransport::new(), MockTransport::new(), domain_tag, lease_duration);
        protocol.run_builtin_endpoints();
        protocol.send_metatraffic();
    }


    struct MockTransportDetect{
        multicast_locator_list: Vec<Locator>,
        unicast_locator_list: Vec<Locator>,
        to_read: RefCell<Vec<(crate::RtpsMessage, Locator)>>
    }

    impl MockTransportDetect{
        fn new() -> Self {            
            Self {
                multicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
                unicast_locator_list: vec![Locator::new_udpv4(7400, [235,0,0,1])],
                to_read: RefCell::new(vec![])
            }
        }
    }

    impl Transport for MockTransportDetect {
        fn write(&self, message: crate::RtpsMessage, _destination_locator: &Locator) {
            println!("{:?}", message);
        }

        fn read(&self) -> crate::transport::TransportResult<Option<(crate::RtpsMessage, Locator)>> {
            Ok(self.to_read.borrow_mut().pop())
        }

        fn unicast_locator_list(&self) -> &Vec<Locator> {
            &self.unicast_locator_list
        }

        fn multicast_locator_list(&self) -> &Vec<Locator> {
            &self.multicast_locator_list
        }
    }


  
    #[test]
    fn spdp_detect() {      

        let domain_id = 0;
        let domain_tag = "".to_string();
        let lease_duration = Duration::from_millis(100);
        let transport = MockTransportDetect::new();

        let locator = Locator::new_udpv4(7401, [127,0,0,1]);
        let participant_guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8 ,9, 10, 11, 12];

        let remote_participant_guid_prefix = [2; 12];
        let unicast_locator_list = vec![Locator::new_udpv4(7401, [127,0,0,1])];
        let multicast_locator_list = vec![Locator::new_udpv4(7401, [127,0,0,1])];
        let expected = SPDPdiscoveredParticipantData::new(
            0,
            "".to_string(), 
            PROTOCOL_VERSION_2_4, 
            remote_participant_guid_prefix, 
            VENDOR_ID, 
            unicast_locator_list.clone(), 
            multicast_locator_list.clone(), 
            unicast_locator_list.clone(),
            multicast_locator_list.clone(),
            BuiltInEndpointSet::new(0),
            Duration::from_millis(100),
        );
        let mut parameter_list = ParameterList::new();
        parameter_list.push(StatusInfo([0,0,0,0]));
        parameter_list.push(KeyHash(expected.key()));
        let inline_qos = Some(parameter_list);
        let data_submessage = Data::new(Endianness::LittleEndian, ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 0, inline_qos, Payload::Data(expected.data(CdrEndianness::LittleEndian)));
        let message = RtpsMessage::new(
            PROTOCOL_VERSION_2_4,
            VENDOR_ID,
            participant_guid_prefix, vec![RtpsSubmessage::Data(data_submessage)]);


        transport.to_read.borrow_mut().push((message, locator));

        let protocol = RtpsProtocol::new(domain_id, MockTransportDetect::new(), transport, domain_tag, lease_duration);
        protocol.receive_metatraffic();
        protocol.run_builtin_endpoints();
        let builtin_subscriber = protocol.participant.builtin_subscriber().lock().unwrap();
        let mut first_endpoint = builtin_subscriber.endpoints().into_iter().next().unwrap().lock().unwrap();

        assert!(first_endpoint.guid().entity_id() == ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR);       
            
        let spdp_detector = first_endpoint.get_mut::<StatelessReader>().unwrap();

        let cache = spdp_detector.reader_cache();
        let cc = cache.changes().iter().next().unwrap();        
        let result = SPDPdiscoveredParticipantData::from_key_data( cc.instance_handle(), cc.data_value(), 0);
        assert!(result == expected)

        // SEDP builtinendpoint to be configured according the received cache change
    }
}