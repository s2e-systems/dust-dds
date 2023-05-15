use crate::{
    builtin_topics::ParticipantBuiltinTopicData,
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
    infrastructure::{error::DdsResult, time::Duration},
    topic_definition::type_support::{
        DdsDeserialize, DdsSerialize, DdsSerializedKey, DdsType, RepresentationType, PL_CDR_LE, RepresentationFormat,
    },
};

use super::parameter_id_values::{
    DEFAULT_DOMAIN_TAG, DEFAULT_PARTICIPANT_LEASE_DURATION, PID_BUILTIN_ENDPOINT_QOS,
    PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_MULTICAST_LOCATOR, PID_DEFAULT_UNICAST_LOCATOR,
    PID_DOMAIN_ID, PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS, PID_METATRAFFIC_MULTICAST_LOCATOR,
    PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_GUID, PID_PARTICIPANT_LEASE_DURATION,
    PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION, PID_VENDORID,
};

#[derive(
    Debug, PartialEq, Eq, derive_more::Into, derive_more::From, serde::Serialize, serde::Deserialize,
)]
struct DomainTag(String);
impl Default for DomainTag {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG.to_string())
    }
}

#[derive(
    Debug, PartialEq, Eq, derive_more::From, derive_more::Into, serde::Serialize, serde::Deserialize,
)]
struct LeaseDuration(Duration);
impl Default for LeaseDuration {
    fn default() -> Self {
        Self(DEFAULT_PARTICIPANT_LEASE_DURATION)
    }
}

pub const DCPS_PARTICIPANT: &str = "DCPSParticipant";

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParticipantProxy {
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

impl ParticipantProxy {
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
    ) -> Self {
        Self {
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
        }
    }

    pub fn domain_id(&self) -> i32 {
        self.domain_id
    }

    pub fn domain_tag(&self) -> &str {
        &self.domain_tag.0
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

    pub fn _expects_inline_qos(&self) -> ExpectsInlineQos {
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

impl DdsSerialize for ParticipantProxy {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

    fn dds_serialize_parameter_list<W: std::io::Write>(
        &self,
        serializer: &mut ParameterListSerializer<W>,
    ) -> DdsResult<()> {
        serializer.serialize_parameter(PID_DOMAIN_ID, &self.domain_id)?;
        serializer.serialize_parameter_if_not_default(PID_DOMAIN_TAG, &self.domain_tag)?;
        serializer.serialize_parameter(PID_PROTOCOL_VERSION, &self.protocol_version)?;
        // guid_prefix omitted as of Table 9.10 - Omitted Builtin Endpoint Parameters
        serializer.serialize_parameter(PID_VENDORID, &self.vendor_id)?;
        serializer
            .serialize_parameter_if_not_default(PID_EXPECTS_INLINE_QOS, &self.expects_inline_qos)?;
        serializer.serialize_parameter_vector(
            PID_METATRAFFIC_UNICAST_LOCATOR,
            &self.metatraffic_unicast_locator_list,
        )?;
        serializer.serialize_parameter_vector(
            PID_METATRAFFIC_MULTICAST_LOCATOR,
            &self.metatraffic_multicast_locator_list,
        )?;
        serializer.serialize_parameter_vector(
            PID_DEFAULT_UNICAST_LOCATOR,
            &self.default_unicast_locator_list,
        )?;
        serializer.serialize_parameter_vector(
            PID_DEFAULT_MULTICAST_LOCATOR,
            &self.default_multicast_locator_list,
        )?;
        serializer
            .serialize_parameter(PID_BUILTIN_ENDPOINT_SET, &self.available_builtin_endpoints)?;
        serializer.serialize_parameter_if_not_default(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            &self.manual_liveliness_count,
        )?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        serializer.serialize_parameter_if_not_default(
            PID_BUILTIN_ENDPOINT_QOS,
            &self.builtin_endpoint_qos,
        )
    }
}

impl<'de> DdsDeserialize<'de> for ParticipantProxy {
    fn dds_deserialize_parameter_list<E: byteorder::ByteOrder>(
        deserializer: &mut ParameterListDeserializer<'de, E>,
    ) -> DdsResult<Self> {
        Ok(Self {
            domain_id: deserializer.get(PID_DOMAIN_ID)?,
            domain_tag: deserializer.get_or_default(PID_DOMAIN_TAG)?,
            protocol_version: deserializer.get(PID_PROTOCOL_VERSION)?,
            guid_prefix: deserializer.get(PID_PARTICIPANT_GUID)?,
            vendor_id: deserializer.get(PID_VENDORID)?,
            expects_inline_qos: deserializer.get_or_default(PID_EXPECTS_INLINE_QOS)?,
            metatraffic_unicast_locator_list: deserializer
                .get_list(PID_METATRAFFIC_UNICAST_LOCATOR)?,
            metatraffic_multicast_locator_list: deserializer
                .get_list(PID_METATRAFFIC_MULTICAST_LOCATOR)?,
            default_unicast_locator_list: deserializer.get_list(PID_DEFAULT_UNICAST_LOCATOR)?,
            default_multicast_locator_list: deserializer.get_list(PID_DEFAULT_MULTICAST_LOCATOR)?,
            available_builtin_endpoints: deserializer.get(PID_BUILTIN_ENDPOINT_SET)?,
            manual_liveliness_count: deserializer
                .get_or_default(PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT)?,
            // Default value is a deviation from the standard and is used for interoperability reasons:
            builtin_endpoint_qos: deserializer.get_or_default(PID_BUILTIN_ENDPOINT_QOS)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpdpDiscoveredParticipantData {
    dds_participant_data: ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    lease_duration: LeaseDuration,
}
impl RepresentationFormat for SpdpDiscoveredParticipantData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
}

impl SpdpDiscoveredParticipantData {
    pub fn new(
        dds_participant_data: ParticipantBuiltinTopicData,
        participant_proxy: ParticipantProxy,
        lease_duration: Duration,
    ) -> Self {
        Self {
            dds_participant_data,
            participant_proxy,
            lease_duration: lease_duration.into(),
        }
    }

    pub fn dds_participant_data(&self) -> &ParticipantBuiltinTopicData {
        &self.dds_participant_data
    }

    pub fn participant_proxy(&self) -> &ParticipantProxy {
        &self.participant_proxy
    }

    pub fn _lease_duration(&self) -> &Duration {
        &self.lease_duration.0
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
        self.dds_participant_data.key().value.as_ref().into()
    }

    fn set_key_fields_from_serialized_key(&mut self, _key: &DdsSerializedKey) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

impl<'de> DdsDeserialize<'de> for SpdpDiscoveredParticipantData {
    fn dds_deserialize_parameter_list<E: byteorder::ByteOrder>(
        deserializer: &mut ParameterListDeserializer<'de, E>,
    ) -> DdsResult<Self> {
        Ok(Self {
            dds_participant_data: DdsDeserialize::dds_deserialize_parameter_list(deserializer)?,
            participant_proxy: DdsDeserialize::dds_deserialize_parameter_list(deserializer)?,
            lease_duration: deserializer.get(PID_PARTICIPANT_LEASE_DURATION)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_topics::BuiltInTopicKey;
    use crate::implementation::parameter_list_serde::serde_parameter_list_deserializer::dds_deserialize;
    use crate::implementation::parameter_list_serde::serde_parameter_list_serializer::dds_serialize;
    use crate::implementation::rtps::types::{LocatorAddress, LocatorKind, LocatorPort};
    use crate::infrastructure::qos_policy::UserDataQosPolicy;

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.dds_serialize(&mut writer).unwrap();
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
        let result: SpdpDiscoveredParticipantData =
            dds_deserialize(&mut data).unwrap();
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
        let result = dds_serialize(&data).unwrap();
        assert_eq!(result, expected);
    }
}
