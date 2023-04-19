use crate::{
    builtin_topics::{BuiltInTopicKey, ParticipantBuiltinTopicData},
    domain::domain_participant_factory::DomainId,
    implementation::{
        parameter_list_serde::{
            parameter_list_deserializer::ParameterListDeserializer,
            parameter_list_serializer::ParameterListSerializer,
        },
        rtps::{
            discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
            types::{Count, ExpectsInlineQos, GuidPrefix, Locator, ProtocolVersion, VendorId},
        },
    },
    infrastructure::{error::DdsResult, qos_policy::UserDataQosPolicy, time::Duration},
    topic_definition::type_support::{
        DdsDeserialize, DdsSerialize, DdsSerializedKey, DdsType, Endianness,
    },
};

use super::parameter_id_values::{
    DEFAULT_DOMAIN_TAG, DEFAULT_PARTICIPANT_LEASE_DURATION, PID_BUILTIN_ENDPOINT_QOS,
    PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_MULTICAST_LOCATOR, PID_DEFAULT_UNICAST_LOCATOR,
    PID_DOMAIN_ID, PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS, PID_METATRAFFIC_MULTICAST_LOCATOR,
    PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_GUID, PID_PARTICIPANT_LEASE_DURATION,
    PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION, PID_USER_DATA, PID_VENDORID,
};

#[derive(
    Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, derive_more::Into, derive_more::From,
)]
struct DomainTag(String);
impl Default for DomainTag {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG.to_string())
    }
}

#[derive(
    Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, derive_more::From, derive_more::Into,
)]
struct LeaseDuration(Duration);
impl Default for LeaseDuration {
    fn default() -> Self {
        Self(DEFAULT_PARTICIPANT_LEASE_DURATION)
    }
}

pub const DCPS_PARTICIPANT: &str = "DCPSParticipant";

#[derive(Debug, PartialEq, Eq)]
struct ParticipantProxy {
    domain_id: DomainId,
    domain_tag: DomainTag,
    protocol_version: ProtocolVersion,
    guid_prefix: GuidPrefix,
    vendor_id: VendorId,
    expects_inline_qos: ExpectsInlineQos,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    available_builtin_endpoints: BuiltinEndpointSet,
    manual_liveliness_count: Count,
    builtin_endpoint_qos: BuiltinEndpointQos,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SpdpDiscoveredParticipantData {
    dds_participant_data: ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    lease_duration: LeaseDuration,
}

impl SpdpDiscoveredParticipantData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
        key: BuiltInTopicKey,
        user_data: UserDataQosPolicy,
        lease_duration: Duration,
    ) -> Self {
        Self {
            dds_participant_data: ParticipantBuiltinTopicData { key, user_data },
            participant_proxy: ParticipantProxy {
                domain_id,
                domain_tag: domain_tag.into(),
                protocol_version,
                guid_prefix,
                vendor_id,
                expects_inline_qos: expects_inline_qos.into(),
                metatraffic_unicast_locator_list,
                metatraffic_multicast_locator_list,
                default_unicast_locator_list,
                default_multicast_locator_list,
                available_builtin_endpoints,
                manual_liveliness_count,
                builtin_endpoint_qos,
            },
            lease_duration: lease_duration.into(),
        }
    }

    pub fn dds_participant_data(&self) -> &ParticipantBuiltinTopicData {
        &self.dds_participant_data
    }

    pub fn domain_id(&self) -> DomainId {
        self.participant_proxy.domain_id
    }

    pub fn domain_tag(&self) -> &str {
        &self.participant_proxy.domain_tag.0
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.participant_proxy.guid_prefix
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        &self.participant_proxy.default_unicast_locator_list
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        &self.participant_proxy.default_multicast_locator_list
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        &self.participant_proxy.metatraffic_unicast_locator_list
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &self.participant_proxy.metatraffic_multicast_locator_list
    }

    pub fn available_builtin_endpoints(&self) -> BuiltinEndpointSet {
        self.participant_proxy.available_builtin_endpoints
    }
}

impl DdsType for SpdpDiscoveredParticipantData {
    fn type_name() -> &'static str {
        "SpdpDiscoveredParticipantData"
    }

    fn has_key() -> bool {
        true
    }

    fn get_serialized_key(&self) -> DdsSerializedKey {
        self.dds_participant_data.key.value.as_ref().into()
    }

    fn set_key_fields_from_serialized_key(&mut self, _key: &DdsSerializedKey) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

impl DdsSerialize for SpdpDiscoveredParticipantData {
    fn serialize<W: std::io::Write, E: Endianness>(&self, writer: W) -> DdsResult<()> {
        let mut parameter_list_serializer = ParameterListSerializer::<_, E>::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        parameter_list_serializer
            .serialize_parameter(PID_DOMAIN_ID, &self.participant_proxy.domain_id)?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_DOMAIN_TAG,
            &self.participant_proxy.domain_tag,
        )?;
        parameter_list_serializer.serialize_parameter(
            PID_PROTOCOL_VERSION,
            &self.participant_proxy.protocol_version,
        )?;
        // guid_prefix omitted as of Table 9.10 - Omitted Builtin Endpoint Parameters
        parameter_list_serializer
            .serialize_parameter(PID_PARTICIPANT_GUID, &self.dds_participant_data.key)?;
        parameter_list_serializer
            .serialize_parameter(PID_VENDORID, &self.participant_proxy.vendor_id)?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_EXPECTS_INLINE_QOS,
            &self.participant_proxy.expects_inline_qos,
        )?;
        parameter_list_serializer.serialize_parameter_vector(
            PID_METATRAFFIC_UNICAST_LOCATOR,
            &self.participant_proxy.metatraffic_unicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector(
            PID_METATRAFFIC_MULTICAST_LOCATOR,
            &self.participant_proxy.metatraffic_multicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector(
            PID_DEFAULT_UNICAST_LOCATOR,
            &self.participant_proxy.default_unicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector(
            PID_DEFAULT_MULTICAST_LOCATOR,
            &self.participant_proxy.default_multicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter(
            PID_BUILTIN_ENDPOINT_SET,
            &self.participant_proxy.available_builtin_endpoints,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            &self.participant_proxy.manual_liveliness_count,
        )?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_BUILTIN_ENDPOINT_QOS,
            &self.participant_proxy.builtin_endpoint_qos,
        )?;
        parameter_list_serializer
            .serialize_parameter(PID_PARTICIPANT_LEASE_DURATION, &self.lease_duration)?;
        parameter_list_serializer.serialize_parameter_if_not_default(
            PID_USER_DATA,
            &self.dds_participant_data.user_data,
        )?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl<'de> DdsDeserialize<'de> for SpdpDiscoveredParticipantData {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        let domain_id = param_list.get(PID_DOMAIN_ID)?;
        let domain_tag = param_list.get_or_default(PID_DOMAIN_TAG)?;
        let protocol_version = param_list.get(PID_PROTOCOL_VERSION)?;
        let vendor_id = param_list.get(PID_VENDORID)?;
        let expects_inline_qos = param_list.get_or_default(PID_EXPECTS_INLINE_QOS)?;
        let metatraffic_unicast_locator_list =
            param_list.get_list(PID_METATRAFFIC_UNICAST_LOCATOR)?;
        let metatraffic_multicast_locator_list =
            param_list.get_list(PID_METATRAFFIC_MULTICAST_LOCATOR)?;
        let default_unicast_locator_list = param_list.get_list(PID_DEFAULT_UNICAST_LOCATOR)?;
        let default_multicast_locator_list = param_list.get_list(PID_DEFAULT_MULTICAST_LOCATOR)?;
        let available_builtin_endpoints = param_list.get(PID_BUILTIN_ENDPOINT_SET)?;
        // Default value is a deviation from the standard and is used for interoperability reasons
        let manual_liveliness_count =
            param_list.get_or_default(PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT)?;
        let builtin_endpoint_qos = param_list.get_or_default(PID_BUILTIN_ENDPOINT_QOS)?;
        let key = param_list.get(PID_PARTICIPANT_GUID)?;
        let user_data = param_list.get_or_default(PID_USER_DATA)?;
        let lease_duration = param_list.get(PID_PARTICIPANT_LEASE_DURATION)?;

        let dds_participant_data = ParticipantBuiltinTopicData { key, user_data };
        let v = dds_participant_data.key.value;
        let guid_prefix = GuidPrefix::new([
            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11],
        ]);

        Ok(Self {
            dds_participant_data,
            participant_proxy: ParticipantProxy {
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
            },
            lease_duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::types::{LocatorAddress, LocatorKind, LocatorPort};
    use crate::infrastructure::qos_policy::UserDataQosPolicy;
    use crate::topic_definition::type_support::LittleEndian;

    pub fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.serialize::<_, LittleEndian>(&mut writer).unwrap();
        writer
    }

    #[test]
    fn deserialize_spdp_discovered_participant_data() {
        let locator1 = Locator::new(
            LocatorKind::new(11),
            LocatorPort::new(12),
            LocatorAddress::new([1; 16]),
        );
        let locator2 = Locator::new(
            LocatorKind::new(21),
            LocatorPort::new(22),
            LocatorAddress::new([2; 16]),
        );

        let domain_id = 1;
        let domain_tag = "ab".to_string();
        let protocol_version = ProtocolVersion::new(2, 4);
        let guid_prefix = GuidPrefix::new([8; 12]);
        let vendor_id = VendorId::new([73, 74]);
        let expects_inline_qos = true;
        let metatraffic_unicast_locator_list = vec![locator1, locator2];
        let metatraffic_multicast_locator_list = vec![locator1];
        let default_unicast_locator_list = vec![locator1];
        let default_multicast_locator_list = vec![locator1];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count::new(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );
        let lease_duration = Duration::new(10, 11);

        let expected = SpdpDiscoveredParticipantData::new(
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
            BuiltInTopicKey {
                value: [8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 1, 0xc1],
            },
            UserDataQosPolicy { value: vec![] },
            lease_duration,
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
        let result: SpdpDiscoveredParticipantData = DdsDeserialize::deserialize(&mut data).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn serialize_spdp_discovered_participant_data() {
        let locator1 = Locator::new(
            LocatorKind::new(11),
            LocatorPort::new(12),
            LocatorAddress::new([1; 16]),
        );
        let locator2 = Locator::new(
            LocatorKind::new(21),
            LocatorPort::new(22),
            LocatorAddress::new([2; 16]),
        );

        let domain_id = 1;
        let domain_tag = "ab".to_string();
        let protocol_version = ProtocolVersion::new(2, 4);
        let guid_prefix = GuidPrefix::new([8; 12]);
        let vendor_id = VendorId::new([73, 74]);
        let expects_inline_qos = true.into();
        let metatraffic_unicast_locator_list = vec![locator1, locator2];
        let metatraffic_multicast_locator_list = vec![locator1];
        let default_unicast_locator_list = vec![locator1];
        let default_multicast_locator_list = vec![locator1];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count::new(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );
        let lease_duration = Duration::new(10, 11);

        let data = SpdpDiscoveredParticipantData::new(
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
            BuiltInTopicKey {
                value: [8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 1, 0xc1],
            },
            UserDataQosPolicy { value: vec![] },
            lease_duration,
        );

        let expected = vec![
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
            0, 0, 1, 0xc1, // EntityId
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
        assert_eq!(to_bytes_le(&data), expected);
    }
}
