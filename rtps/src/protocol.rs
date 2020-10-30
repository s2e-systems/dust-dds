use crate::transport::Transport;
use crate::behavior::types::Duration;
use crate::discovery::spdp::{SimpleEndpointDiscoveryProtocol, SPDPdiscoveredParticipantData};
use crate::endpoint_types::BuiltInEndpointSet;
use crate::structure::RtpsParticipant;
use crate::structure::entity::RtpsEntity;

use rust_dds_interface::types::DomainId;

pub struct RtpsProtocol {
    participant: RtpsParticipant,
    spdp: SimpleEndpointDiscoveryProtocol,
}

impl RtpsProtocol {
    pub fn new(domain_id: DomainId, userdata_transport: impl Transport, metatraffic_transport: impl Transport, domain_tag: String, lease_duration: Duration) -> Self {

        let participant = RtpsParticipant::new(domain_id, userdata_transport, metatraffic_transport);

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
        let spdp = SimpleEndpointDiscoveryProtocol::new(data);

        participant.builtin_publisher().lock().unwrap().mut_endpoints().push(spdp.spdp_builtin_participant_writer().clone());

        Self {
            participant,
            spdp,
        }
    }
}