use crate::{
    builtin_topics::ParticipantBuiltinTopicData,
    domain::domain_participant_factory::DomainId,
    infrastructure::{error::DdsResult, instance::InstanceHandle, time::Duration},
    rtps::{
        discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
        messages::types::Count,
        types::{GuidPrefix, Locator, ProtocolVersion, VendorId},
    },
    serialized_payload::{
        cdr::{
            deserialize::CdrDeserialize, deserializer::CdrDeserializer, serialize::CdrSerialize,
            serializer::CdrSerializer,
        },
        parameter_list::{
            deserialize::ParameterListDeserialize, serialize::ParameterListSerialize,
        },
    },
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};

use super::parameter_id_values::{
    DEFAULT_DOMAIN_TAG, DEFAULT_EXPECTS_INLINE_QOS, PID_BUILTIN_ENDPOINT_QOS,
    PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_MULTICAST_LOCATOR, PID_DEFAULT_UNICAST_LOCATOR,
    PID_DISCOVERED_PARTICIPANT, PID_DOMAIN_ID, PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS,
    PID_METATRAFFIC_MULTICAST_LOCATOR, PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_GUID,
    PID_PARTICIPANT_LEASE_DURATION, PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION,
    PID_VENDORID,
};

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize)]
struct DomainTag(String);
impl Default for DomainTag {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG.to_string())
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
struct DomainIdParameter(Option<DomainId>);
impl CdrSerialize for DomainIdParameter {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        self.0
            .expect("Default DomainId not supposed to be serialized")
            .serialize(serializer)
    }
}

impl<'de> CdrDeserialize<'de> for DomainIdParameter {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> Result<Self, std::io::Error> {
        // None should not happen since this is only deserialized if the
        // corresponding PID is found
        Ok(Self(Some(CdrDeserialize::deserialize(deserializer)?)))
    }
}

pub const DCPS_PARTICIPANT: &str = "DCPSParticipant";

#[derive(Debug, PartialEq, Eq, Clone, ParameterListSerialize, ParameterListDeserialize)]
pub struct ParticipantProxy {
    #[parameter(id=PID_DOMAIN_ID, default=Default::default())]
    domain_id: DomainIdParameter,
    #[parameter(id=PID_DOMAIN_TAG, default=Default::default())]
    domain_tag: DomainTag,
    #[parameter(id=PID_PROTOCOL_VERSION)]
    protocol_version: ProtocolVersion,
    #[parameter(id=PID_PARTICIPANT_GUID, skip_serialize)]
    guid_prefix: GuidPrefix,
    #[parameter(id=PID_VENDORID)]
    vendor_id: VendorId,
    #[parameter(id=PID_EXPECTS_INLINE_QOS, default=DEFAULT_EXPECTS_INLINE_QOS)]
    expects_inline_qos: bool,
    #[parameter(id=PID_METATRAFFIC_UNICAST_LOCATOR, collection)]
    metatraffic_unicast_locator_list: Vec<Locator>,
    #[parameter(id=PID_METATRAFFIC_MULTICAST_LOCATOR, collection)]
    metatraffic_multicast_locator_list: Vec<Locator>,
    #[parameter(id=PID_DEFAULT_UNICAST_LOCATOR, collection)]
    default_unicast_locator_list: Vec<Locator>,
    #[parameter(id=PID_DEFAULT_MULTICAST_LOCATOR, collection)]
    default_multicast_locator_list: Vec<Locator>,
    #[parameter(id=PID_BUILTIN_ENDPOINT_SET)]
    available_builtin_endpoints: BuiltinEndpointSet,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    #[parameter(id=PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, default=Default::default())]
    manual_liveliness_count: Count,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    #[parameter(id=PID_BUILTIN_ENDPOINT_QOS, default=Default::default())]
    builtin_endpoint_qos: BuiltinEndpointQos,
}

impl ParticipantProxy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_id: Option<DomainId>,
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
    ) -> Self {
        Self {
            domain_id: DomainIdParameter(domain_id),
            domain_tag: DomainTag(domain_tag),
            protocol_version,
            guid_prefix,
            vendor_id,
            expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            available_builtin_endpoints,
            manual_liveliness_count,
            builtin_endpoint_qos,
        }
    }

    pub fn domain_id(&self) -> Option<DomainId> {
        self.domain_id.0
    }

    pub fn domain_tag(&self) -> &str {
        self.domain_tag.0.as_ref()
    }

    pub fn _protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }

    pub fn _vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn _expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_unicast_locator_list.as_ref()
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_multicast_locator_list.as_ref()
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.default_unicast_locator_list.as_ref()
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.default_multicast_locator_list.as_ref()
    }

    pub fn available_builtin_endpoints(&self) -> BuiltinEndpointSet {
        self.available_builtin_endpoints
    }

    pub fn _manual_liveliness_count(&self) -> Count {
        self.manual_liveliness_count
    }

    pub fn _builtin_endpoint_qos(&self) -> BuiltinEndpointQos {
        self.builtin_endpoint_qos
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    DdsSerialize,
    DdsDeserialize,
    ParameterListSerialize,
    ParameterListDeserialize,
)]
#[dust_dds(format = "PL_CDR_LE")]
pub struct SpdpDiscoveredParticipantData {
    dds_participant_data: ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    // Default (DEFAULT_PARTICIPANT_LEASE_DURATION) is ommited compared to the standard due to interoperability reasons
    #[parameter(id = PID_PARTICIPANT_LEASE_DURATION)]
    lease_duration: Duration,
    #[parameter(id = PID_DISCOVERED_PARTICIPANT, collection)]
    discovered_participant_list: Vec<InstanceHandle>,
}

impl SpdpDiscoveredParticipantData {
    pub fn new(
        dds_participant_data: ParticipantBuiltinTopicData,
        participant_proxy: ParticipantProxy,
        lease_duration: Duration,
        discovered_participant_list: Vec<InstanceHandle>,
    ) -> Self {
        Self {
            dds_participant_data,
            participant_proxy,
            lease_duration,
            discovered_participant_list,
        }
    }

    pub fn dds_participant_data(&self) -> &ParticipantBuiltinTopicData {
        &self.dds_participant_data
    }

    pub fn participant_proxy(&self) -> &ParticipantProxy {
        &self.participant_proxy
    }

    pub fn _lease_duration(&self) -> &Duration {
        &self.lease_duration
    }

    pub fn discovered_participant_list(&self) -> &[InstanceHandle] {
        &self.discovered_participant_list
    }
}

impl DdsHasKey for SpdpDiscoveredParticipantData {
    const HAS_KEY: bool = true;
}

impl DdsKey for SpdpDiscoveredParticipantData {
    type Key = [u8; 16];

    fn get_key(&self) -> DdsResult<Self::Key> {
        Ok(self.dds_participant_data.key().value)
    }

    fn get_key_from_serialized_data(serialized_foo: &[u8]) -> DdsResult<Self::Key> {
        Ok(Self::deserialize_data(serialized_foo)?
            .dds_participant_data
            .key()
            .value)
    }
}

impl DdsTypeXml for SpdpDiscoveredParticipantData {
    fn get_type_xml() -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builtin_topics::BuiltInTopicKey, infrastructure::qos_policy::UserDataQosPolicy};

    #[test]
    fn deserialize_spdp_discovered_participant_data() {
        let locator1 = Locator::new(11, 12, [1; 16]);
        let locator2 = Locator::new(21, 22, [2; 16]);

        let domain_id = 1;
        let domain_tag = "ab".to_string();
        let protocol_version = ProtocolVersion::new(2, 4);
        let guid_prefix = [8; 12];
        let vendor_id = [73, 74];
        let expects_inline_qos = true;
        let metatraffic_unicast_locator_list = vec![locator1, locator2];
        let metatraffic_multicast_locator_list = vec![locator1];
        let default_unicast_locator_list = vec![locator1];
        let default_multicast_locator_list = vec![locator1];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = 2;
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );
        let lease_duration = Duration::new(10, 11);

        let expected = SpdpDiscoveredParticipantData::new(
            ParticipantBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 1, 0xc1],
                },
                UserDataQosPolicy { value: vec![] },
            ),
            ParticipantProxy::new(
                Some(domain_id),
                domain_tag,
                protocol_version,
                guid_prefix,
                vendor_id,
                expects_inline_qos,
                metatraffic_unicast_locator_list,
                metatraffic_multicast_locator_list,
                default_unicast_locator_list,
                default_multicast_locator_list,
                available_builtin_endpoints,
                manual_liveliness_count,
                builtin_endpoint_qos,
            ),
            lease_duration,
            vec![],
        );

        let mut data = &[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x15, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion
            0x50, 0x00, 16, 0x00, // PID_PARTICIPANT_GUID, Length
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            0, 0, 1, 0xc1, // EntityId,
            0x16, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            0x43, 0x00, 0x04, 0x00, // PID_EXPECTS_INLINE_QOS, Length: 4,
            0x01, 0x00, 0x00, 0x00, // True
            0x32, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x32, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR
            21, 0x00, 0x00, 0x00, // Locator{kind
            22, 0x00, 0x00, 0x00, // port,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x33, 0x00, 24, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x31, 0x00, 24, 0x00, // PID_DEFAULT_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x48, 0x00, 24, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x58, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET
            0x02, 0x00, 0x00, 0x00, //
            0x34, 0x00, 4, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT
            0x02, 0x00, 0x00, 0x00, // Count
            0x77, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_QOS
            0x00, 0x00, 0x00, 0x20, //
            0x02, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION
            10, 0x00, 0x00, 0x00, // Duration: seconds
            11, 0x00, 0x00, 0x00, // Duration: fraction
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ][..];
        let result = SpdpDiscoveredParticipantData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn serialize_spdp_discovered_participant_data() {
        let locator1 = Locator::new(11, 12, [1; 16]);
        let locator2 = Locator::new(21, 22, [2; 16]);

        let domain_id = Some(1);
        let domain_tag = "ab".to_string();
        let protocol_version = ProtocolVersion::new(2, 4);
        let guid_prefix = [8; 12];
        let vendor_id = [73, 74];
        let expects_inline_qos = true;
        let metatraffic_unicast_locator_list = vec![locator1, locator2];
        let metatraffic_multicast_locator_list = vec![locator1];
        let default_unicast_locator_list = vec![locator1];
        let default_multicast_locator_list = vec![locator1];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = 2;
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );
        let lease_duration = Duration::new(10, 11);

        let data = SpdpDiscoveredParticipantData::new(
            ParticipantBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: [8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 1, 0xc1],
                },
                UserDataQosPolicy { value: vec![] },
            ),
            ParticipantProxy::new(
                domain_id,
                domain_tag,
                protocol_version,
                guid_prefix,
                vendor_id,
                expects_inline_qos,
                metatraffic_unicast_locator_list,
                metatraffic_multicast_locator_list,
                default_unicast_locator_list,
                default_multicast_locator_list,
                available_builtin_endpoints,
                manual_liveliness_count,
                builtin_endpoint_qos,
            ),
            lease_duration,
            vec![],
        );

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x50, 0x00, 16, 0x00, // PID_PARTICIPANT_GUID, Length
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            0, 0, 1, 0xc1, // EntityId
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x15, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion
            0x16, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            0x43, 0x00, 0x04, 0x00, // PID_EXPECTS_INLINE_QOS, Length: 4,
            0x01, 0x00, 0x00, 0x00, // True
            0x32, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x32, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR
            21, 0x00, 0x00, 0x00, // Locator{kind
            22, 0x00, 0x00, 0x00, // port,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x33, 0x00, 24, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x31, 0x00, 24, 0x00, // PID_DEFAULT_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x48, 0x00, 24, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x58, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET
            0x02, 0x00, 0x00, 0x00, //
            0x34, 0x00, 4, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT
            0x02, 0x00, 0x00, 0x00, // Count
            0x77, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_QOS
            0x00, 0x00, 0x00, 0x20, //
            0x02, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION
            10, 0x00, 0x00, 0x00, // Duration: seconds
            11, 0x00, 0x00, 0x00, // Duration: fraction
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        let result = data.serialize_data().unwrap();
        assert_eq!(result, expected);
    }
}
