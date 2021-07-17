use crate::{
    builtin_endpoints::parameterid_list::PID_DOMAIN_ID,
    parameter_list::{ParameterListUdp, ParameterUdp},
    submessage_elements::{
        CountUdp, EntityIdUdp, GuidPrefixUdp, LocatorUdp, ProtocolVersionUdp, VendorIdUdp,
    },
};
use rust_rtps_pim::{
    behavior::types::Duration,
    discovery::{
        spdp::spdp_discovered_participant_data::SPDPdiscoveredParticipantData,
        types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId},
    },
    messages::{
        submessage_elements::{
            CountSubmessageElementType, EntityIdSubmessageElementType,
            GuidPrefixSubmessageElementType, ProtocolVersionSubmessageElementType,
            VendorIdSubmessageElementType,
        },
        types::Count,
    },
    structure::types::{
        EntityId, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_PARTICIPANT,
    },
};

use super::parameterid_list::{
    PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_MULTICAST_LOCATOR, PID_DEFAULT_UNICAST_LOCATOR,
    PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS, PID_METATRAFFIC_MULTICAST_LOCATOR,
    PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_GUID, PID_PARTICIPANT_LEASE_DURATION,
    PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION, PID_VENDORID,
};

const PL_CDR_LE: [u8; 4] = [0x00, 0x03, 0x00, 0x00];

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

impl From<&Locator> for LocatorUdp {
    fn from(value: &Locator) -> Self {
        LocatorUdp {
            kind: value.kind().clone(),
            port: value.port().clone().into(),
            address: value.address().clone().into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DurationUdp {
    pub seconds: i32,
    pub fraction: u32,
}

impl From<Duration> for DurationUdp {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds,
            fraction: value.fraction,
        }
    }
}

struct ParticipantProxy {
    domain_id: DomainId,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    guid_prefix: GuidPrefix,
    vendor_id: VendorId,
    expects_inline_qos: bool,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    available_builtin_endpoints: BuiltinEndpointSet,
    manual_liveliness_count: Count,
    builtin_endpoint_qos: BuiltinEndpointQos,
}

pub struct SPDPdiscoveredParticipantDataUdp {
    // ddsParticipantData: DDS::ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    lease_duration: Duration,
}

impl SPDPdiscoveredParticipantDataUdp {
    pub fn new(
        domain_id: &DomainId,
        domain_tag: &str,
        protocol_version: &ProtocolVersion,
        guid_prefix: &GuidPrefix,
        vendor_id: &VendorId,
        expects_inline_qos: &bool,
        metatraffic_unicast_locator_list: &[Locator],
        metatraffic_multicast_locator_list: &[Locator],
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        available_builtin_endpoints: &BuiltinEndpointSet,
        manual_liveliness_count: &Count,
        builtin_endpoint_qos: &BuiltinEndpointQos,
        lease_duration: &Duration,
    ) -> Self {
        Self {
            participant_proxy: ParticipantProxy {
                domain_id: *domain_id,
                domain_tag: domain_tag.to_owned(),
                protocol_version: *protocol_version,
                guid_prefix: *guid_prefix,
                vendor_id: *vendor_id,
                expects_inline_qos: *expects_inline_qos,
                metatraffic_unicast_locator_list: metatraffic_unicast_locator_list.to_vec(),
                metatraffic_multicast_locator_list: metatraffic_multicast_locator_list.to_vec(),
                default_unicast_locator_list: default_unicast_locator_list.to_vec(),
                default_multicast_locator_list: default_multicast_locator_list.to_vec(),
                available_builtin_endpoints: *available_builtin_endpoints,
                manual_liveliness_count: *manual_liveliness_count,
                builtin_endpoint_qos: *builtin_endpoint_qos,
            },
            lease_duration: *lease_duration,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, rust_serde_cdr::error::Error> {
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

        parameter.push(ParameterUdp::new(
            PID_EXPECTS_INLINE_QOS,
            rust_serde_cdr::to_bytes(&self.participant_proxy.expects_inline_qos).unwrap(),
        ));

        for metatraffic_unicast_locator in &self.participant_proxy.metatraffic_unicast_locator_list
        {
            let value: LocatorUdp = metatraffic_unicast_locator.into();
            parameter.push(ParameterUdp::new(
                PID_METATRAFFIC_UNICAST_LOCATOR,
                rust_serde_cdr::to_bytes(&value).unwrap(),
            ));
        }

        for metatraffic_multicast_locator in
            &self.participant_proxy.metatraffic_multicast_locator_list
        {
            let value: LocatorUdp = metatraffic_multicast_locator.into();
            parameter.push(ParameterUdp::new(
                PID_METATRAFFIC_MULTICAST_LOCATOR,
                rust_serde_cdr::to_bytes(&value).unwrap(),
            ));
        }

        for default_unicast_locator in &self.participant_proxy.default_unicast_locator_list {
            let value: LocatorUdp = default_unicast_locator.into();
            parameter.push(ParameterUdp::new(
                PID_DEFAULT_UNICAST_LOCATOR,
                rust_serde_cdr::to_bytes(&value).unwrap(),
            ));
        }

        for default_multicast_locator in &self.participant_proxy.default_multicast_locator_list {
            let value: LocatorUdp = default_multicast_locator.into();
            parameter.push(ParameterUdp::new(
                PID_DEFAULT_MULTICAST_LOCATOR,
                rust_serde_cdr::to_bytes(&value).unwrap(),
            ));
        }

        parameter.push(ParameterUdp::new(
            PID_BUILTIN_ENDPOINT_SET,
            rust_serde_cdr::to_bytes(&self.participant_proxy.available_builtin_endpoints.0)
                .unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            rust_serde_cdr::to_bytes(&CountUdp::new(
                &self.participant_proxy.manual_liveliness_count,
            ))
            .unwrap(),
        ));

        let value: DurationUdp = self.lease_duration.into();
        parameter.push(ParameterUdp::new(
            PID_PARTICIPANT_LEASE_DURATION,
            rust_serde_cdr::to_bytes(&value).unwrap(),
        ));

        let mut bytes = PL_CDR_LE.to_vec();
        rust_serde_cdr::serialize_into(&ParameterListUdp { parameter }, &mut bytes)
            .unwrap();
        Ok(bytes)
    }
}

impl SPDPdiscoveredParticipantData for SPDPdiscoveredParticipantDataUdp {
    type LocatorListType = Vec<Locator>;

    fn domain_id(&self) -> DomainId {
        self.participant_proxy.domain_id
    }

    fn domain_tag(&self) -> &str {
        &self.participant_proxy.domain_tag
    }

    fn protocol_version(&self) -> ProtocolVersion {
        self.participant_proxy.protocol_version
    }

    fn guid_prefix(&self) -> GuidPrefix {
        self.participant_proxy.guid_prefix
    }

    fn vendor_id(&self) -> VendorId {
        self.participant_proxy.vendor_id
    }

    fn expects_inline_qos(&self) -> bool {
        self.participant_proxy.expects_inline_qos
    }

    fn metatraffic_unicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .metatraffic_unicast_locator_list
            .clone()
    }

    fn metatraffic_multicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .metatraffic_multicast_locator_list
            .clone()
    }

    fn default_unicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy.default_unicast_locator_list.clone()
    }

    fn default_multicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .default_multicast_locator_list
            .clone()
    }

    fn available_builtin_endpoints(&self) -> BuiltinEndpointSet {
        self.participant_proxy.available_builtin_endpoints
    }

    fn manual_liveliness_count(&self) -> Count {
        self.participant_proxy.manual_liveliness_count
    }

    fn builtin_endpoint_qos(&self) -> BuiltinEndpointQos {
        self.participant_proxy.builtin_endpoint_qos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn serialize_complete_spdp_discovered_participant_data() {
        let locator1 = Locator::new(1, 1, [1; 16]);
        let locator2 = Locator::new(2, 2, [2; 16]);

        let domain_id = 1;
        let domain_tag = "abc";
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid_prefix = [1; 12];
        let vendor_id = [9, 9];
        let expects_inline_qos = true;
        let metatraffic_unicast_locator_list = &[locator1, locator2];
        let metatraffic_multicast_locator_list = &[locator1, locator2];
        let default_unicast_locator_list = &[locator1, locator2];
        let default_multicast_locator_list = &[locator1, locator2];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );
        let lease_duration = Duration {
            seconds: 10,
            fraction: 0,
        };

        let spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
            &domain_id,
            domain_tag,
            &protocol_version,
            &guid_prefix,
            &vendor_id,
            &expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            &available_builtin_endpoints,
            &manual_liveliness_count,
            &builtin_endpoint_qos,
            &lease_duration,
        );

        let serialized_data = spdp_discovered_participant_data.to_bytes().unwrap();
        let expected_data = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId(1)
            0x14, 0x40, 0x04, 0x00, // PID_DOMAIN_TAG, Length: 4
            b'a', b'b', b'c', 0x00, // DomainTag('abc')
            0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
            0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
            0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
            0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
            0x43, 0x00, 0x04, 0x00, // PID_EXPECTS_INLINE_QOS, Length: 4,
            0x01, 0x00, 0x00, 0x00, // True
            0x32, 0x00, 0x18, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR, Length: 24,
            0x01, 0x00, 0x00, 0x00, // Locator{kind:1
            0x01, 0x00, 0x00, 0x00, // port:1,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address: [1;16]
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x32, 0x00, 0x18, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR, Length: 24,
            0x02, 0x00, 0x00, 0x00, // Locator{kind:2
            0x02, 0x00, 0x00, 0x00, // port:2,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address: [2;16]
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x33, 0x00, 0x18, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR, Length: 24,
            0x01, 0x00, 0x00, 0x00, // Locator{kind:1
            0x01, 0x00, 0x00, 0x00, // port:1,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address: [1;16]
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x33, 0x00, 0x18, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR, Length: 24,
            0x02, 0x00, 0x00, 0x00, // Locator{kind:2
            0x02, 0x00, 0x00, 0x00, // port:2,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address: [2;16]
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x31, 0x00, 0x18, 0x00, // PID_DEFAULT_UNICAST_LOCATOR, Length: 24,
            0x01, 0x00, 0x00, 0x00, // Locator{kind:1
            0x01, 0x00, 0x00, 0x00, // port:1,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address: [1;16]
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x31, 0x00, 0x18, 0x00, // PID_DEFAULT_UNICAST_LOCATOR, Length: 24,
            0x02, 0x00, 0x00, 0x00, // Locator{kind:2
            0x02, 0x00, 0x00, 0x00, // port:2,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address: [2;16]
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x48, 0x00, 0x18, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length: 24,
            0x01, 0x00, 0x00, 0x00, // Locator{kind:1
            0x01, 0x00, 0x00, 0x00, // port:1,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address: [1;16]
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x48, 0x00, 0x18, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length: 24,
            0x02, 0x00, 0x00, 0x00, // Locator{kind:2
            0x02, 0x00, 0x00, 0x00, // port:2,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address: [2;16]
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
            0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
            0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
            0x02, 0x00, 0x00, 0x00, // Count(2)
            0x02, 0x00, 0x08, 0x00, // PID_PARTICIPANT_LEASE_DURATION, Length: 8
            0x0a, 0x00, 0x00, 0x00, // Duration{seconds:30,
            0x00, 0x00, 0x00, 0x00, //          fraction:0}
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ];

        assert_eq!(serialized_data, expected_data);
    }
}
