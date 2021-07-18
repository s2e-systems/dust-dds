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
    structure::types::{GuidPrefix, Locator, ProtocolVersion, VendorId, GUID},
};

use super::parameterid_list::{
    PID_BUILTIN_ENDPOINT_QOS, PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_MULTICAST_LOCATOR,
    PID_DEFAULT_UNICAST_LOCATOR, PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS,
    PID_METATRAFFIC_MULTICAST_LOCATOR, PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_GUID,
    PID_PARTICIPANT_LEASE_DURATION, PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION,
    PID_VENDORID,
};

const PL_CDR_LE: [u8; 4] = [0x00, 0x03, 0x00, 0x00];

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct GUIDUdp {
    prefix: GuidPrefixUdp,
    entity_id: EntityIdUdp,
}

impl GUIDUdp {
    pub fn new(guid: &GUID) -> Self {
        Self {
            prefix: GuidPrefixUdp::new(guid.prefix()),
            entity_id: EntityIdUdp::new(guid.entity_id()),
        }
    }

    pub fn value(&self) -> GUID {
        GUID::new(self.prefix.value(), self.entity_id.value())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DurationUdp {
    seconds: i32,
    fraction: u32,
}

impl DurationUdp {
    pub fn new(duration: &Duration) -> Self {
        Self {
            seconds: duration.seconds,
            fraction: duration.fraction,
        }
    }

    pub fn value(&self) -> Duration {
        Duration {
            seconds: self.seconds,
            fraction: self.fraction,
        }
    }
}

#[derive(PartialEq, Debug)]
struct ParticipantProxy {
    domain_id: u32,
    domain_tag: String,
    protocol_version: ProtocolVersionUdp,
    guid: GUIDUdp,
    vendor_id: VendorIdUdp,
    expects_inline_qos: bool,
    metatraffic_unicast_locator_list: Vec<LocatorUdp>,
    metatraffic_multicast_locator_list: Vec<LocatorUdp>,
    default_unicast_locator_list: Vec<LocatorUdp>,
    default_multicast_locator_list: Vec<LocatorUdp>,
    available_builtin_endpoints: u32,
    manual_liveliness_count: CountUdp,
    builtin_endpoint_qos: u32,
}

#[derive(PartialEq, Debug)]
pub struct SPDPdiscoveredParticipantDataUdp {
    // ddsParticipantData: DDS::ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    lease_duration: DurationUdp,
}

impl SPDPdiscoveredParticipantDataUdp {
    // const DEFAULT_LEASE_DURATION: DurationUdp = DurationUdp {
    //     seconds: 30,
    //     fraction: 0,
    // };

    const DEFAULT_EXPECTS_INLINE_QOS: bool = false;

    pub fn new(
        domain_id: &DomainId,
        domain_tag: &str,
        protocol_version: &ProtocolVersion,
        guid: &GUID,
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
                protocol_version: ProtocolVersionUdp::new(protocol_version),
                guid: GUIDUdp::new(guid),
                vendor_id: VendorIdUdp::new(vendor_id),
                expects_inline_qos: *expects_inline_qos,
                metatraffic_unicast_locator_list: metatraffic_unicast_locator_list
                    .iter()
                    .map(|x| LocatorUdp::new(x))
                    .collect(),
                metatraffic_multicast_locator_list: metatraffic_multicast_locator_list
                    .iter()
                    .map(|x| LocatorUdp::new(x))
                    .collect(),
                default_unicast_locator_list: default_unicast_locator_list
                    .iter()
                    .map(|x| LocatorUdp::new(x))
                    .collect(),
                default_multicast_locator_list: default_multicast_locator_list
                    .iter()
                    .map(|x| LocatorUdp::new(x))
                    .collect(),
                available_builtin_endpoints: available_builtin_endpoints.0,
                manual_liveliness_count: CountUdp::new(manual_liveliness_count),
                builtin_endpoint_qos: builtin_endpoint_qos.0,
            },
            lease_duration: DurationUdp::new(lease_duration),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, rust_serde_cdr::error::Error> {
        let mut parameter = Vec::new();

        parameter.push(ParameterUdp::new(
            PID_DOMAIN_ID,
            rust_serde_cdr::serializer::to_bytes(&self.participant_proxy.domain_id).unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_DOMAIN_TAG,
            rust_serde_cdr::serializer::to_bytes(&self.participant_proxy.domain_tag).unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_PROTOCOL_VERSION,
            rust_serde_cdr::serializer::to_bytes(&self.participant_proxy.protocol_version).unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_PARTICIPANT_GUID,
            rust_serde_cdr::serializer::to_bytes(&self.participant_proxy.guid).unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_VENDORID,
            rust_serde_cdr::serializer::to_bytes(&self.participant_proxy.vendor_id).unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_EXPECTS_INLINE_QOS,
            rust_serde_cdr::serializer::to_bytes(&self.participant_proxy.expects_inline_qos)
                .unwrap(),
        ));

        for metatraffic_unicast_locator in &self.participant_proxy.metatraffic_unicast_locator_list
        {
            parameter.push(ParameterUdp::new(
                PID_METATRAFFIC_UNICAST_LOCATOR,
                rust_serde_cdr::serializer::to_bytes(&metatraffic_unicast_locator).unwrap(),
            ));
        }

        for metatraffic_multicast_locator in
            &self.participant_proxy.metatraffic_multicast_locator_list
        {
            parameter.push(ParameterUdp::new(
                PID_METATRAFFIC_MULTICAST_LOCATOR,
                rust_serde_cdr::serializer::to_bytes(&metatraffic_multicast_locator).unwrap(),
            ));
        }

        for default_unicast_locator in &self.participant_proxy.default_unicast_locator_list {
            parameter.push(ParameterUdp::new(
                PID_DEFAULT_UNICAST_LOCATOR,
                rust_serde_cdr::serializer::to_bytes(&default_unicast_locator).unwrap(),
            ));
        }

        for default_multicast_locator in &self.participant_proxy.default_multicast_locator_list {
            parameter.push(ParameterUdp::new(
                PID_DEFAULT_MULTICAST_LOCATOR,
                rust_serde_cdr::serializer::to_bytes(&default_multicast_locator).unwrap(),
            ));
        }

        parameter.push(ParameterUdp::new(
            PID_BUILTIN_ENDPOINT_SET,
            rust_serde_cdr::serializer::to_bytes(
                &self.participant_proxy.available_builtin_endpoints,
            )
            .unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            rust_serde_cdr::serializer::to_bytes(&self.participant_proxy.manual_liveliness_count)
                .unwrap(),
        ));

        parameter.push(ParameterUdp::new(
            PID_PARTICIPANT_LEASE_DURATION,
            rust_serde_cdr::serializer::to_bytes(&self.lease_duration).unwrap(),
        ));

        let mut bytes = PL_CDR_LE.to_vec();
        rust_serde_cdr::serializer::serialize_into(&ParameterListUdp { parameter }, &mut bytes)
            .unwrap();
        Ok(bytes)
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, rust_serde_cdr::error::Error> {
        let _representation: [u8; 4] = rust_serde_cdr::deserializer::from_bytes(&buf[0..4])?;
        let parameter_list: ParameterListUdp = rust_serde_cdr::deserializer::from_bytes(&buf[4..])?;

        let domain_id = parameter_list.get(PID_DOMAIN_ID).unwrap();
        let domain_tag = match parameter_list
            .parameter
            .iter()
            .find(|x| x.parameter_id == PID_DOMAIN_TAG)
        {
            Some(parameter) => String::from_utf8(parameter.value.0.clone())
                .unwrap()
                .trim_end_matches(char::from(0))
                .to_owned(),
            None => String::new(),
        };
        let protocol_version: ProtocolVersionUdp =
            parameter_list.get(PID_PROTOCOL_VERSION).unwrap();
        let guid: GUIDUdp = parameter_list.get(PID_PARTICIPANT_GUID).unwrap();
        let vendor_id: VendorIdUdp = parameter_list.get(PID_VENDORID).unwrap();
        let expects_inline_qos = parameter_list
            .get(PID_EXPECTS_INLINE_QOS)
            .unwrap_or(Self::DEFAULT_EXPECTS_INLINE_QOS);
        let metatraffic_unicast_locator_list =
            parameter_list.get_list(PID_METATRAFFIC_UNICAST_LOCATOR);
        let metatraffic_multicast_locator_list =
            parameter_list.get_list(PID_METATRAFFIC_MULTICAST_LOCATOR);
        let default_unicast_locator_list = parameter_list.get_list(PID_DEFAULT_UNICAST_LOCATOR);
        let default_multicast_locator_list = parameter_list.get_list(PID_DEFAULT_MULTICAST_LOCATOR);
        let available_builtin_endpoints = parameter_list.get(PID_BUILTIN_ENDPOINT_SET).unwrap();

        let manual_liveliness_count = parameter_list
            .get(PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT)
            .unwrap_or(CountUdp(0));
        let builtin_endpoint_qos = parameter_list.get(PID_BUILTIN_ENDPOINT_QOS).unwrap_or(0);

        let lease_duration = parameter_list.get(PID_PARTICIPANT_LEASE_DURATION).unwrap();

        let participant_proxy = ParticipantProxy {
            domain_id,
            domain_tag,
            protocol_version,
            guid,
            vendor_id,
            expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            available_builtin_endpoints,
            manual_liveliness_count,
            builtin_endpoint_qos,
        };

        Ok(Self {
            participant_proxy: participant_proxy,
            lease_duration: lease_duration,
        })
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
        self.participant_proxy.protocol_version.value()
    }

    fn guid_prefix(&self) -> GuidPrefix {
        *self.participant_proxy.guid.value().prefix()
    }

    fn vendor_id(&self) -> VendorId {
        self.participant_proxy.vendor_id.value()
    }

    fn expects_inline_qos(&self) -> bool {
        self.participant_proxy.expects_inline_qos
    }

    fn metatraffic_unicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .metatraffic_unicast_locator_list
            .iter()
            .map(|x| x.value())
            .collect()
    }

    fn metatraffic_multicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .metatraffic_multicast_locator_list
            .iter()
            .map(|x| x.value())
            .collect()
    }

    fn default_unicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .default_unicast_locator_list
            .iter()
            .map(|x| x.value())
            .collect()
    }

    fn default_multicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .default_multicast_locator_list
            .iter()
            .map(|x| x.value())
            .collect()
    }

    fn available_builtin_endpoints(&self) -> BuiltinEndpointSet {
        BuiltinEndpointSet(self.participant_proxy.available_builtin_endpoints)
    }

    fn manual_liveliness_count(&self) -> Count {
        self.participant_proxy.manual_liveliness_count.value()
    }

    fn builtin_endpoint_qos(&self) -> BuiltinEndpointQos {
        BuiltinEndpointQos(self.participant_proxy.builtin_endpoint_qos)
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::ENTITYID_PARTICIPANT;

    use super::*;

    #[test]
    pub fn serialize_complete_spdp_discovered_participant_data() {
        let locator1 = Locator::new(1, 1, [1; 16]);
        let locator2 = Locator::new(2, 2, [2; 16]);

        let domain_id = 1;
        let domain_tag = "abc";
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid = GUID::new([1; 12], ENTITYID_PARTICIPANT);
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
            &guid,
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
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 4
            0x04, 0x00, 0x00, 0x00, // DomainTag(length: 4)
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

    #[test]
    fn deserialize_complete_spdp_discovered_participant_data() {
        #[rustfmt::skip]
        let spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::from_bytes(&[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 4, 0x00,    // PID_DOMAIN_ID, Length
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 4, 0x00,    // PID_DOMAIN_TAG, Length
            b'a', b'b', b'c', 0x00, // DomainTag
            0x15, 0x00, 4, 0x00,    // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion: major, minor
            0x50, 0x00, 16, 0x00,   // PID_PARTICIPANT_GUID, Length
            0x01, 0x01, 0x01, 0x01, // GuidPrefix
            0x01, 0x01, 0x01, 0x01, // GuidPrefix
            0x01, 0x01, 0x01, 0x01, // GuidPrefix
            0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
            0x16, 0x00, 4, 0x00,    // PID_VENDORID, Length
            0x09, 0x09, 0x00, 0x00, // VendorId
            0x43, 0x00, 4, 0x00,    // PID_EXPECTS_INLINE_QOS, Length
            0x01, 0x00, 0x00, 0x00, // True
            0x32, 0x00, 24, 0x00,   // PID_METATRAFFIC_UNICAST_LOCATOR, Length
            0x01, 0x00, 0x00, 0x00, // Locator: kind
            0x01, 0x00, 0x00, 0x00, // Locator: port
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x32, 0x00, 24, 0x00,   // PID_METATRAFFIC_UNICAST_LOCATOR, Length
            0x02, 0x00, 0x00, 0x00, // Locator: kind
            0x02, 0x00, 0x00, 0x00, // Locator: port
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x33, 0x00, 24, 0x00,   // PID_METATRAFFIC_MULTICAST_LOCATOR, Length
            0x01, 0x00, 0x00, 0x00, // Locator: kind
            0x01, 0x00, 0x00, 0x00, // Locator: port
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x33, 0x00, 24, 0x00,   // PID_METATRAFFIC_MULTICAST_LOCATOR, Length
            0x02, 0x00, 0x00, 0x00, // Locator: kind
            0x02, 0x00, 0x00, 0x00, // Locator: port,
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x31, 0x00, 24, 0x00,   // PID_DEFAULT_UNICAST_LOCATOR, Length
            0x01, 0x00, 0x00, 0x00, // Locator: kind
            0x01, 0x00, 0x00, 0x00, // Locator: port
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x31, 0x00, 24, 0x00,   // PID_DEFAULT_UNICAST_LOCATOR, Length
            0x02, 0x00, 0x00, 0x00, // Locator: kind
            0x02, 0x00, 0x00, 0x00, // Locator: port
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x48, 0x00, 24, 0x00,   // PID_DEFAULT_MULTICAST_LOCATOR, Length
            0x01, 0x00, 0x00, 0x00, // Locator: kind
            0x01, 0x00, 0x00, 0x00, // Locator: port
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x48, 0x00, 024, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length,
            0x02, 0x00, 0x00, 0x00, // Locator: kind
            0x02, 0x00, 0x00, 0x00, // Locator: port
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x58, 0x00, 4, 0x00,    // PID_BUILTIN_ENDPOINT_SET, Length
            0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
            0x34, 0x00, 4, 0x00,    // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length
            0x02, 0x00, 0x00, 0x00, // Count
            0x02, 0x00, 8, 0x00,    // PID_PARTICIPANT_LEASE_DURATION, Length
            10, 0x00, 0x00, 0x00, // Duration: seconds
            0x00, 0x00, 0x00, 0x00, // Duration: fraction
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length
        ]).unwrap();

        let locator1 = Locator::new(1, 1, [1; 16]);
        let locator2 = Locator::new(2, 2, [2; 16]);

        let domain_id = 1;
        let domain_tag = "abc";
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid = GUID::new([1; 12], ENTITYID_PARTICIPANT);
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

        let expected_spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
            &domain_id,
            domain_tag,
            &protocol_version,
            &guid,
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

        assert_eq!(
            spdp_discovered_participant_data,
            expected_spdp_discovered_participant_data
        );
    }
}
