use rust_rtps_pim::{
    behavior::types::Duration,
    discovery::{
        spdp::spdp_discovered_participant_data::SPDPdiscoveredParticipantData,
        types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId},
    },
    messages::{
        submessage_elements::{
            EntityIdSubmessageElementType, GuidPrefixSubmessageElementType,
            ProtocolVersionSubmessageElementType, VendorIdSubmessageElementType,
        },
        types::Count,
    },
    structure::types::{
        EntityId, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_PARTICIPANT,
    },
};
use serde::ser::SerializeStruct;

use crate::{
    builtin_endpoints::parameterid_list::PID_DOMAIN_ID,
    parameter_list::{ParameterListUdp, ParameterUdp},
    submessage_elements::{EntityIdUdp, GuidPrefixUdp, ProtocolVersionUdp, VendorIdUdp},
};

use super::parameterid_list::{
    PID_DOMAIN_TAG, PID_PARTICIPANT_GUID, PID_PROTOCOL_VERSION, PID_VENDORID,
};

const PL_CDR_LE: [u8; 4] = [0x00, 0x03, 0x00, 0x00];

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct GUIDUdp {
    pub prefix: GuidPrefixUdp,
    pub entity_id: EntityIdUdp,
}

impl GUIDUdp {
    pub fn new(prefix: &GuidPrefix, entity_id: &EntityId) -> Self {
        Self {
            prefix: GuidPrefixUdp::new(prefix),
            entity_id: EntityIdUdp::new(entity_id),
        }
    }
}

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

        parameter.push(ParameterUdp::new(
            PID_DOMAIN_ID,
            rust_serde_cdr::to_bytes(&self.participant_proxy.domain_id).unwrap(),
        ));
        parameter.push(ParameterUdp::new(
            PID_DOMAIN_TAG,
            rust_serde_cdr::to_bytes(&self.participant_proxy.domain_tag).unwrap(),
        ));
        parameter.push(ParameterUdp::new(
            PID_PROTOCOL_VERSION,
            rust_serde_cdr::to_bytes(&ProtocolVersionUdp::new(
                &self.participant_proxy.protocol_version,
            ))
            .unwrap(),
        ));

        let guid = GUIDUdp::new(&self.participant_proxy.guid_prefix, &ENTITYID_PARTICIPANT);
        parameter.push(ParameterUdp::new(
            PID_PARTICIPANT_GUID,
            rust_serde_cdr::to_bytes(&guid).unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_VENDORID,
            rust_serde_cdr::to_bytes(&VendorIdUdp::new(&self.participant_proxy.vendor_id)).unwrap(),
        ));

        let mut state = serializer.serialize_struct("SPDPdiscoveredParticipantData", 2)?;
        state.serialize_field("representation", &PL_CDR_LE)?;
        state.serialize_field("value", &ParameterListUdp { parameter })?;
        state.end()
    }
}

struct ParticipantProxy {
    domain_id: DomainId,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    guid_prefix: GuidPrefix,
    vendor_id: VendorId,
}

impl SPDPdiscoveredParticipantData for SPDPdiscoveredParticipantDataUdp {
    type LocatorListType = Vec<Locator>;

    fn new(
        domain_id: &DomainId,
        domain_tag: &str,
        protocol_version: &ProtocolVersion,
        guid_prefix: &GuidPrefix,
        vendor_id: &VendorId,
    ) -> Self {
        Self {
            participant_proxy: ParticipantProxy {
                domain_id: *domain_id,
                domain_tag: domain_tag.to_owned(),
                protocol_version: *protocol_version,
                guid_prefix: *guid_prefix,
                vendor_id: *vendor_id,
            },
            _lease_duration: Duration(0),
        }
    }

    fn domain_id(&self) -> DomainId {
        self.participant_proxy.domain_id
    }

    fn domain_tag(&self) -> &str {
        &self.participant_proxy.domain_tag
    }

    fn protocol_version(&self) -> ProtocolVersion {
        todo!()
    }

    fn guid_prefix(&self) -> GuidPrefix {
        self.participant_proxy.guid_prefix
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
