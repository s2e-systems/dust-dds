use rust_rtps_pim::{
    behavior::types::Duration,
    discovery::{
        spdp::spdp_discovered_participant_data::SPDPdiscoveredParticipantData,
        types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId},
    },
    messages::types::Count,
    structure::types::{GuidPrefix, Locator, ProtocolVersion, VendorId},
};
use serde::ser::SerializeStruct;

use crate::parameter_list::{ParameterListUdp, ParameterUdp};

const PL_CDR_LE: [u8; 4] = [0x00, 0x03, 0x00, 0x00];

pub struct SPDPdiscoveredParticipantDataUdp {
    // ddsParticipantData: DDS::ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    _lease_duration: Duration,
}

impl serde::Serialize for SPDPdiscoveredParticipantDataUdp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut parameter = Vec::new();

        parameter.push(ParameterUdp::new(10, vec![1, 2, 3, 4]));

        let mut state = serializer.serialize_struct("SPDPdiscoveredParticipantData", 2)?;
        state.serialize_field("representation", &PL_CDR_LE)?;
        state.serialize_field("value", &ParameterListUdp { parameter })?;
        state.end()
    }
}

struct ParticipantProxy {
    domain_id: DomainId,
}

impl SPDPdiscoveredParticipantData for SPDPdiscoveredParticipantDataUdp {
    type LocatorListType = Vec<Locator>;

    fn new(domain_id: DomainId) -> Self {
        Self {
            participant_proxy: ParticipantProxy { domain_id },
            _lease_duration: Duration(0),
        }
    }

    fn domain_id(&self) -> DomainId {
        self.participant_proxy.domain_id
    }

    fn domain_tag(&self) -> &str {
        todo!()
    }

    fn protocol_version(&self) -> ProtocolVersion {
        todo!()
    }

    fn guid_prefix(&self) -> GuidPrefix {
        todo!()
    }

    fn vendor_id(&self) -> VendorId {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        todo!()
    }

    fn metatraffic_unicast_locator_list(&self) -> Self::LocatorListType {
        todo!()
    }

    fn metatraffic_multicast_locator_list(&self) -> Self::LocatorListType {
        todo!()
    }

    fn default_unicast_locator_list(&self) -> Self::LocatorListType {
        todo!()
    }

    fn default_multicast_locator_list(&self) -> Self::LocatorListType {
        todo!()
    }

    fn available_builtin_endpoints(&self) -> BuiltinEndpointSet {
        todo!()
    }

    fn manual_liveliness_count(&self) -> Count {
        todo!()
    }

    fn builtin_endpoint_qos(&self) -> BuiltinEndpointQos {
        todo!()
    }
}
