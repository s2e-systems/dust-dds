use std::sync::Arc;
use crate::transport::Transport;
use crate::behavior::types::Duration;
use crate::discovery::spdp::{SimpleParticipantDiscoveryProtocol, SPDPdiscoveredParticipantData};
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
        let spdp = SimpleParticipantDiscoveryProtocol::new(data);

        participant.builtin_publisher().lock().unwrap().mut_endpoints().push(spdp.spdp_builtin_participant_writer().clone());
        participant.builtin_subscriber().lock().unwrap().mut_endpoints().push(spdp.spdp_builtin_participant_reader().clone());

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
        self.spdp.spdp_builtin_participant_reader().lock().unwrap().run(
            |_| println!("Discovery data")
        );

        self.spdp.spdp_builtin_participant_writer().lock().unwrap().run();
    }

    // pub fn receive_metatraffic(&self) {
    //     RtpsMessageReceiver::receive(
    //         self.participant.guid().prefix(), 
    //         self.participant.metatraffic_transport().as_ref(),
    //         &[self.participant.builtin_publisher(), self.participant.builtin_subscriber()])
    // }

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
    use crate::types::Locator;

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
}