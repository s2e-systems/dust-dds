use crate::{
    builtin_topics::ParticipantBuiltinTopicData,
    cdr::{
        deserialize::CdrDeserialize,
        deserializer::CdrDeserializer,
        error::CdrResult,
        parameter_list_deserialize::ParameterListDeserialize,
        parameter_list_deserializer::ParameterListDeserializer,
        parameter_list_serialize::ParameterListSerialize,
        parameter_list_serializer::ParameterListSerializer,
        serialize::CdrSerialize,
        serializer::CdrSerializer,
    },
    domain::domain_participant_factory::DomainId,
    implementation::rtps::{
        discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
        messages::types::Count,
        types::{GuidPrefix, Locator, ProtocolVersion, VendorId},
    },
    infrastructure::{error::DdsResult, time::Duration},
    topic_definition::type_support::{
        DdsDeserialize, DdsGetKeyFromFoo, DdsGetKeyFromSerializedData, DdsHasKey, DdsSerializedKey,
    },
};

use super::parameter_id_values::{
    DEFAULT_DOMAIN_TAG, DEFAULT_EXPECTS_INLINE_QOS, DEFAULT_PARTICIPANT_LEASE_DURATION,
    PID_BUILTIN_ENDPOINT_QOS, PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_MULTICAST_LOCATOR,
    PID_DEFAULT_UNICAST_LOCATOR, PID_DOMAIN_ID, PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS,
    PID_METATRAFFIC_MULTICAST_LOCATOR, PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_GUID,
    PID_PARTICIPANT_LEASE_DURATION, PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION,
    PID_VENDORID,
};

#[derive(
    Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::AsRef, CdrSerialize, CdrDeserialize,
)]
struct DomainTag(String);
impl Default for DomainTag {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG.to_string())
    }
}
#[derive(
    Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::AsRef, CdrSerialize, CdrDeserialize,
)]
struct ExpectsInlineQos(bool);
impl Default for ExpectsInlineQos {
    fn default() -> Self {
        Self(DEFAULT_EXPECTS_INLINE_QOS)
    }
}
#[derive(
    Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::AsRef, CdrSerialize, CdrDeserialize,
)]
struct LeaseDuration(Duration);
impl Default for LeaseDuration {
    fn default() -> Self {
        Self(DEFAULT_PARTICIPANT_LEASE_DURATION)
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, derive_more::From, derive_more::AsRef)]
struct DomainIdParameter(Option<DomainId>);
impl CdrSerialize for DomainIdParameter {
    fn serialize(&self, serializer: &mut CdrSerializer) -> CdrResult<()> {
        self.0
            .expect("Default DomainId not supposed to be serialized")
            .serialize(serializer)
    }
}

impl<'de> CdrDeserialize<'de> for DomainIdParameter {
    fn deserialize(deserializer: &mut CdrDeserializer<'de>) -> CdrResult<Self> {
        // None should not happen since this is only deserialized if the
        // corresponding PID is found
        Ok(Self(Some(CdrDeserialize::deserialize(deserializer)?)))
    }
}

pub const DCPS_PARTICIPANT: &str = "DCPSParticipant";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParticipantProxy {
    domain_id: DomainIdParameter, //ParameterWithDefault<PID_DOMAIN_ID, DomainIdParameter>,
    domain_tag: DomainTag,        //ParameterWithDefault<PID_DOMAIN_TAG, DomainTag>,
    protocol_version: ProtocolVersion, //Parameter<PID_PROTOCOL_VERSION, ProtocolVersion>
    guid_prefix: GuidPrefix,      //Parameter<PID_PARTICIPANT_GUID, GuidPrefix>
    vendor_id: VendorId,          //  Parameter<PID_VENDORID, VendorId>
    expects_inline_qos: ExpectsInlineQos, //ParameterWithDefault<PID_EXPECTS_INLINE_QOS, ExpectsInlineQos>,
    metatraffic_unicast_locator_list: Vec<Locator>, //ParameterVector<PID_METATRAFFIC_UNICAST_LOCATOR, Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>, //ParameterVector<PID_METATRAFFIC_MULTICAST_LOCATOR, Locator>,
    default_unicast_locator_list: Vec<Locator>, // ParameterVector<PID_DEFAULT_UNICAST_LOCATOR, Locator>,
    default_multicast_locator_list: Vec<Locator>, //ParameterVector<PID_DEFAULT_MULTICAST_LOCATOR, Locator>,
    available_builtin_endpoints: BuiltinEndpointSet, //Parameter<PID_BUILTIN_ENDPOINT_SET, BuiltinEndpointSet>,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    manual_liveliness_count: Count, //ParameterWithDefault<PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Count>,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    builtin_endpoint_qos: BuiltinEndpointQos, //ParameterWithDefault<PID_BUILTIN_ENDPOINT_QOS, BuiltinEndpointQos>,
}

impl CdrSerialize for ParticipantProxy {
    fn serialize(&self, serializer: &mut CdrSerializer) -> CdrResult<()> {
        self.domain_id.serialize(serializer)?;
        self.domain_tag.serialize(serializer)?;
        self.protocol_version.serialize(serializer)?;
        // guid_prefix omitted as of Table 9.10 - Omitted Builtin Endpoint Parameters
        self.vendor_id.serialize(serializer)?;
        self.expects_inline_qos.serialize(serializer)?;
        self.metatraffic_unicast_locator_list
            .serialize(serializer)?;
        self.metatraffic_multicast_locator_list
            .serialize(serializer)?;
        self.default_unicast_locator_list.serialize(serializer)?;
        self.default_multicast_locator_list.serialize(serializer)?;
        self.available_builtin_endpoints.serialize(serializer)?;
        self.manual_liveliness_count.serialize(serializer)?;
        self.builtin_endpoint_qos.serialize(serializer)?;
        Ok(())
    }
}

impl<'de> CdrDeserialize<'de> for ParticipantProxy {
    fn deserialize(_deserializer: &mut CdrDeserializer<'de>) -> CdrResult<Self> {
        todo!()
    }
}

impl ParameterListSerialize for ParticipantProxy {
    fn serialize(&self, serializer: &mut ParameterListSerializer) -> Result<(), std::io::Error> {
        serializer.write_with_default(PID_DOMAIN_ID, &self.domain_id, &Default::default())?;
        serializer.write_with_default(PID_DOMAIN_TAG, &self.domain_tag, &Default::default())?;
        serializer.write(PID_PROTOCOL_VERSION, &self.protocol_version)?;
        // guid_prefix omitted as of Table 9.10 - Omitted Builtin Endpoint Parameters
        serializer.write(PID_VENDORID, &self.vendor_id)?;
        serializer.write(PID_EXPECTS_INLINE_QOS, &self.expects_inline_qos)?;
        serializer.write_list_elements(PID_METATRAFFIC_UNICAST_LOCATOR, &self.metatraffic_unicast_locator_list)?;
        serializer.write_list_elements(PID_METATRAFFIC_MULTICAST_LOCATOR, &self.metatraffic_unicast_locator_list)?;
        serializer.write_list_elements(PID_DEFAULT_UNICAST_LOCATOR,  &self.default_unicast_locator_list )?;
        serializer.write_list_elements(PID_DEFAULT_MULTICAST_LOCATOR,  &self.default_multicast_locator_list )?;
        serializer.write(PID_BUILTIN_ENDPOINT_SET, &self.available_builtin_endpoints)?;
        serializer.write_with_default(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            &self.manual_liveliness_count,
            &Default::default(),
        )?;
        serializer.write_with_default(
            PID_BUILTIN_ENDPOINT_QOS,
            &self.builtin_endpoint_qos,
            &Default::default(),
        )?;

        Ok(())
    }
}

impl<'de> ParameterListDeserialize<'de> for ParticipantProxy {
    fn deserialize(
        pl_deserializer: &mut ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            domain_id: pl_deserializer.read_with_default(PID_DOMAIN_ID, Default::default())?,
            domain_tag: pl_deserializer.read_with_default(PID_DOMAIN_TAG, Default::default())?,
            protocol_version: pl_deserializer.read(PID_PROTOCOL_VERSION)?,
            guid_prefix: pl_deserializer.read(PID_PARTICIPANT_GUID)?,
            vendor_id: pl_deserializer.read(PID_VENDORID)?,
            expects_inline_qos: pl_deserializer
                .read_with_default(PID_EXPECTS_INLINE_QOS, Default::default())?,
            metatraffic_unicast_locator_list: pl_deserializer
                .read_all(PID_METATRAFFIC_UNICAST_LOCATOR)?,
            metatraffic_multicast_locator_list: pl_deserializer
                .read_all(PID_METATRAFFIC_MULTICAST_LOCATOR)?,
            default_unicast_locator_list: pl_deserializer.read_all(PID_DEFAULT_UNICAST_LOCATOR)?,
            default_multicast_locator_list: pl_deserializer
                .read_all(PID_DEFAULT_MULTICAST_LOCATOR)?,
            available_builtin_endpoints: pl_deserializer.read(PID_BUILTIN_ENDPOINT_SET)?,
            manual_liveliness_count: pl_deserializer
                .read_with_default(PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Default::default())?,
            builtin_endpoint_qos: pl_deserializer
                .read_with_default(PID_BUILTIN_ENDPOINT_QOS, Default::default())?,
        })
    }
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
            domain_id: DomainIdParameter::from(domain_id).into(),
            domain_tag: DomainTag::from(domain_tag).into(),
            protocol_version: protocol_version.into(),
            guid_prefix: guid_prefix.into(),
            vendor_id: vendor_id.into(),
            expects_inline_qos: ExpectsInlineQos::from(expects_inline_qos).into(),
            metatraffic_unicast_locator_list: metatraffic_unicast_locator_list.into(),
            metatraffic_multicast_locator_list: metatraffic_multicast_locator_list.into(),
            default_unicast_locator_list: default_unicast_locator_list.into(),
            default_multicast_locator_list: default_multicast_locator_list.into(),
            available_builtin_endpoints: available_builtin_endpoints.into(),
            manual_liveliness_count: manual_liveliness_count.into(),
            builtin_endpoint_qos: builtin_endpoint_qos.into(),
        }
    }

    pub fn domain_id(&self) -> Option<DomainId> {
        self.domain_id.as_ref().clone()
    }

    pub fn domain_tag(&self) -> &str {
        self.domain_tag.as_ref().as_ref()
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
        *self.expects_inline_qos.as_ref()
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

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize)]
pub struct SpdpDiscoveredParticipantData {
    dds_participant_data: ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    lease_duration: LeaseDuration,
}

impl<'de> ParameterListDeserialize<'de> for SpdpDiscoveredParticipantData {
    fn deserialize(
        pl_deserializer: &mut ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            dds_participant_data: ParameterListDeserialize::deserialize(pl_deserializer)?,
            participant_proxy: ParameterListDeserialize::deserialize(pl_deserializer)?,
            lease_duration: pl_deserializer
                .read_with_default(PID_PARTICIPANT_LEASE_DURATION, Default::default())?,
        })
    }
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
            lease_duration: LeaseDuration::from(lease_duration).into(),
        }
    }

    pub fn dds_participant_data(&self) -> &ParticipantBuiltinTopicData {
        &self.dds_participant_data
    }

    pub fn participant_proxy(&self) -> &ParticipantProxy {
        &self.participant_proxy
    }

    pub fn _lease_duration(&self) -> &Duration {
        self.lease_duration.as_ref()
    }
}

impl DdsHasKey for SpdpDiscoveredParticipantData {
    const HAS_KEY: bool = true;
}

impl DdsGetKeyFromFoo for SpdpDiscoveredParticipantData {
    fn get_key_from_foo(&self) -> DdsResult<DdsSerializedKey> {
        Ok(self.dds_participant_data.key().value.to_vec().into())
    }
}

impl DdsGetKeyFromSerializedData for SpdpDiscoveredParticipantData {
    fn get_key_from_serialized_data(mut serialized_data: &[u8]) -> DdsResult<DdsSerializedKey> {
        Ok(Self::deserialize_data(&mut serialized_data)?
            .dds_participant_data
            .key()
            .value
            .to_vec()
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey, infrastructure::qos_policy::UserDataQosPolicy,
        topic_definition::type_support::DdsSerializeData,
    };

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
        assert_eq!(result, expected.into());
    }
}
