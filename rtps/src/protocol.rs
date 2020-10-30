use crate::transport::Transport;
use crate::behavior::types::Duration;
use crate::discovery::spdp::{SimpleParticipantDiscoveryProtocol, SPDPdiscoveredParticipantData};
use crate::endpoint_types::BuiltInEndpointSet;
use crate::structure::RtpsParticipant;
use crate::structure::entity::RtpsEntity;
use crate::messages::message_receiver::RtpsMessageReceiver;
use crate::messages::message_sender::RtpsMessageSender;

use rust_dds_interface::types::DomainId;

pub struct RtpsProtocol {
    participant: RtpsParticipant,
    spdp: SimpleParticipantDiscoveryProtocol,
}

impl RtpsProtocol {
    pub fn new(domain_id: DomainId, userdata_transport: impl Transport, metatraffic_transport: impl Transport, domain_tag: String, lease_duration: Duration) -> Self {

        let guid_prefix = [1,2,3,4,5,6,7,8,9,10,11,12];  //TODO: Should be uniquely generated
        let participant = RtpsParticipant::new(domain_id, guid_prefix, userdata_transport, metatraffic_transport);

        let data = SPDPdiscoveredParticipantData::new(
            participant.domain_id(),
            domain_tag, 
            participant.protocol_version(), 
            participant.guid().prefix(), 
            participant.vendor_id(), 
            participant.metatraffic_transport().unicast_locator_list().clone(), 
            participant.metatraffic_transport().multicast_locator_list().clone(), 
            participant.userdata_transport().unicast_locator_list().clone(),
            participant.userdata_transport().multicast_locator_list().clone(),
            BuiltInEndpointSet::new(0),
            lease_duration,
        );
        let spdp = SimpleParticipantDiscoveryProtocol::new(data);

        participant.builtin_publisher().lock().unwrap().mut_endpoints().push(spdp.spdp_builtin_participant_writer().clone());
        participant.builtin_subscriber().lock().unwrap().mut_endpoints().push(spdp.spdp_builtin_participant_reader().clone());

        Self {
            participant,
            spdp,
        }
    }

    pub fn run_builtin_endpoints(&self) {
        self.spdp.spdp_builtin_participant_reader().lock().unwrap().run(
            |_| println!("Discovery data")
        );

        self.spdp.spdp_builtin_participant_writer().lock().unwrap().run();
    }

    pub fn receive_metatraffic(&self) {
        RtpsMessageReceiver::receive(
            self.participant.guid().prefix(), 
            self.participant.metatraffic_transport().as_ref(),
            &[self.participant.builtin_publisher(), self.participant.builtin_subscriber()])
    }

    pub fn send_metatraffic(&self) {
        RtpsMessageSender::send(
            self.participant.guid().prefix(), 
            self.participant.metatraffic_transport().as_ref(),
            &[self.participant.builtin_publisher(), self.participant.builtin_subscriber()])
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn spdp_announce() {

    }
}