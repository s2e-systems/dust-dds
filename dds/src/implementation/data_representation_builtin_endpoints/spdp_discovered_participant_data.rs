use crate::builtin_topics::{BuiltInTopicKey, ParticipantBuiltinTopicData};

use crate::domain::domain_participant_factory::DomainId;
use crate::implementation::rtps::discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet};
use crate::implementation::rtps::types::{
    Count, Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_PARTICIPANT,
};
use crate::infrastructure::error::DdsResult;
use crate::infrastructure::qos_policy::UserDataQosPolicy;
use crate::topic_definition::type_support::LittleEndian;
use crate::{
    implementation::parameter_list_serde::{
        parameter_list_deserializer::ParameterListDeserializer,
        parameter_list_serializer::ParameterListSerializer,
        serde_remote_rtps_pim::{
            DomainTag, DomainTagDeserialize, DomainTagSerialize, ExpectsInlineQosDeserialize,
            ExpectsInlineQosSerialize,
        },
    },
    infrastructure::time::Duration,
    topic_definition::type_support::{DdsDeserialize, DdsSerialize, DdsType, Endianness},
};

use super::parameter_id_values::{
    DEFAULT_PARTICIPANT_LEASE_DURATION, PID_BUILTIN_ENDPOINT_QOS, PID_BUILTIN_ENDPOINT_SET,
    PID_DEFAULT_MULTICAST_LOCATOR, PID_DEFAULT_UNICAST_LOCATOR, PID_DOMAIN_ID, PID_DOMAIN_TAG,
    PID_EXPECTS_INLINE_QOS, PID_METATRAFFIC_MULTICAST_LOCATOR, PID_METATRAFFIC_UNICAST_LOCATOR,
    PID_PARTICIPANT_GUID, PID_PARTICIPANT_LEASE_DURATION, PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
    PID_PROTOCOL_VERSION, PID_USER_DATA, PID_VENDORID,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ParticipantLeaseDuration(Duration);

impl Default for ParticipantLeaseDuration {
    fn default() -> Self {
        Self(DEFAULT_PARTICIPANT_LEASE_DURATION)
    }
}

impl From<Duration> for ParticipantLeaseDuration {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl From<ParticipantLeaseDuration> for Duration {
    fn from(value: ParticipantLeaseDuration) -> Self {
        value.0
    }
}

pub const DCPS_PARTICIPANT: &str = "DCPSParticipant";

#[derive(Debug, PartialEq, Eq)]
pub struct ParticipantProxy {
    pub domain_id: DomainId,
    pub domain_tag: String,
    pub protocol_version: ProtocolVersion,
    pub guid_prefix: GuidPrefix,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub available_builtin_endpoints: BuiltinEndpointSet,
    pub manual_liveliness_count: Count,
    pub builtin_endpoint_qos: BuiltinEndpointQos,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SpdpDiscoveredParticipantData {
    pub dds_participant_data: ParticipantBuiltinTopicData,
    pub participant_proxy: ParticipantProxy,
    pub lease_duration: ParticipantLeaseDuration,
}

impl SpdpDiscoveredParticipantData {
    pub fn domain_id(&self) -> DomainId {
        self.participant_proxy.domain_id
    }

    pub fn domain_tag(&self) -> &str {
        &self.participant_proxy.domain_tag
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.participant_proxy.guid_prefix
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

    fn get_serialized_key<E: Endianness>(&self) -> Vec<u8> {
        self.dds_participant_data.key.value.to_vec()
    }

    fn set_key_fields_from_serialized_key<E: Endianness>(&mut self, _key: &[u8]) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

impl DdsSerialize for SpdpDiscoveredParticipantData {
    fn serialize<W: std::io::Write, E: Endianness>(&self, writer: W) -> DdsResult<()> {
        let guid = Guid::new(self.participant_proxy.guid_prefix, ENTITYID_PARTICIPANT);

        let mut parameter_list_serializer = ParameterListSerializer::<_, E>::new(writer);
        parameter_list_serializer.serialize_payload_header()?;

        parameter_list_serializer
            .serialize_parameter::<&i32, _>(PID_DOMAIN_ID, &self.participant_proxy.domain_id)?;
        parameter_list_serializer.serialize_parameter_if_not_default::<DomainTagSerialize, _>(
            PID_DOMAIN_TAG,
            &DomainTag(self.participant_proxy.domain_tag.as_str()),
        )?;
        parameter_list_serializer.serialize_parameter::<&ProtocolVersion, _>(
            PID_PROTOCOL_VERSION,
            &self.participant_proxy.protocol_version,
        )?;
        parameter_list_serializer.serialize_parameter::<&Guid, _>(PID_PARTICIPANT_GUID, &guid)?;
        parameter_list_serializer
            .serialize_parameter::<&[u8; 2], _>(PID_VENDORID, &self.participant_proxy.vendor_id)?;
        parameter_list_serializer
            .serialize_parameter_if_not_default::<ExpectsInlineQosSerialize, _>(
                PID_EXPECTS_INLINE_QOS,
                &self.participant_proxy.expects_inline_qos,
            )?;
        parameter_list_serializer.serialize_parameter_vector::<&Locator, _>(
            PID_METATRAFFIC_UNICAST_LOCATOR,
            &self.participant_proxy.metatraffic_unicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector::<&Locator, _>(
            PID_METATRAFFIC_MULTICAST_LOCATOR,
            &self.participant_proxy.metatraffic_multicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector::<&Locator, _>(
            PID_DEFAULT_UNICAST_LOCATOR,
            &self.participant_proxy.default_unicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter_vector::<&Locator, _>(
            PID_DEFAULT_MULTICAST_LOCATOR,
            &self.participant_proxy.default_multicast_locator_list,
        )?;
        parameter_list_serializer.serialize_parameter::<&BuiltinEndpointSet, _>(
            PID_BUILTIN_ENDPOINT_SET,
            &self.participant_proxy.available_builtin_endpoints,
        )?;
        parameter_list_serializer.serialize_parameter::<&Count, _>(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            &self.participant_proxy.manual_liveliness_count,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&BuiltinEndpointQos, _>(
            PID_BUILTIN_ENDPOINT_QOS,
            &self.participant_proxy.builtin_endpoint_qos,
        )?;
        parameter_list_serializer.serialize_parameter::<&ParticipantLeaseDuration, _>(
            PID_PARTICIPANT_LEASE_DURATION,
            &self.lease_duration,
        )?;
        parameter_list_serializer.serialize_parameter_if_not_default::<&UserDataQosPolicy, _>(
            PID_USER_DATA,
            &self.dds_participant_data.user_data,
        )?;
        parameter_list_serializer.serialize_sentinel()
    }
}

impl<'de> DdsDeserialize<'de> for SpdpDiscoveredParticipantData {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        let guid = param_list.get::<Guid, Guid>(PID_PARTICIPANT_GUID)?;
        let user_data = param_list.get_or_default::<UserDataQosPolicy, _>(PID_USER_DATA)?;
        let domain_id = param_list.get::<i32, _>(PID_DOMAIN_ID)?;
        let domain_tag = param_list.get_or_default::<DomainTagDeserialize, _>(PID_DOMAIN_TAG)?;
        let protocol_version = param_list.get::<ProtocolVersion, _>(PID_PROTOCOL_VERSION)?;
        let vendor_id = param_list.get::<[u8; 2], _>(PID_VENDORID)?;
        let expects_inline_qos =
            param_list.get_or_default::<ExpectsInlineQosDeserialize, _>(PID_EXPECTS_INLINE_QOS)?;
        let metatraffic_unicast_locator_list =
            param_list.get_list::<Locator, _>(PID_METATRAFFIC_UNICAST_LOCATOR)?;
        let metatraffic_multicast_locator_list =
            param_list.get_list::<Locator, _>(PID_METATRAFFIC_MULTICAST_LOCATOR)?;
        let default_unicast_locator_list =
            param_list.get_list::<Locator, _>(PID_DEFAULT_UNICAST_LOCATOR)?;
        let default_multicast_locator_list =
            param_list.get_list::<Locator, _>(PID_DEFAULT_MULTICAST_LOCATOR)?;
        let available_builtin_endpoints =
            param_list.get::<BuiltinEndpointSet, _>(PID_BUILTIN_ENDPOINT_SET)?;
        let manual_liveliness_count =
            param_list.get::<Count, _>(PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT)?;
        let builtin_endpoint_qos =
            param_list.get_or_default::<BuiltinEndpointQos, _>(PID_BUILTIN_ENDPOINT_QOS)?;
        let lease_duration =
            param_list.get::<ParticipantLeaseDuration, _>(PID_PARTICIPANT_LEASE_DURATION)?;

        Ok(Self {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey { value: guid.into() },
                user_data,
            },
            participant_proxy: ParticipantProxy {
                domain_id,
                domain_tag,
                protocol_version,
                guid_prefix: guid.prefix(),
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

    fn deserialize_key(mut buf: &[u8]) -> DdsResult<Vec<u8>> {
        Ok(Self::deserialize(&mut buf)?.get_serialized_key::<LittleEndian>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::types::{EntityId, EntityKind};
    use crate::infrastructure::qos_policy::UserDataQosPolicy;
    use crate::topic_definition::type_support::LittleEndian;

    pub fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.serialize::<_, LittleEndian>(&mut writer).unwrap();
        writer
    }

    #[test]
    fn deserialize_spdp_discovered_participant_data() {
        let locator1 = Locator::new(11, 12, [1; 16]);
        let locator2 = Locator::new(21, 22, [2; 16]);

        let domain_id = 1;
        let domain_tag = "ab".to_string();
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid_prefix = GuidPrefix::from([8; 12]);
        let guid = Guid::new(guid_prefix, EntityId::new([0, 0, 1], EntityKind::BuiltInParticipant));
        let vendor_id = [73, 74];
        let expects_inline_qos = true;
        let metatraffic_unicast_locator_list = vec![locator1, locator2];
        let metatraffic_multicast_locator_list = vec![locator1];
        let default_unicast_locator_list = vec![locator1];
        let default_multicast_locator_list = vec![locator1];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );

        let dds_participant_data = ParticipantBuiltinTopicData {
            key: BuiltInTopicKey { value: guid.into() },
            user_data: UserDataQosPolicy { value: vec![] },
        };
        let participant_proxy = ParticipantProxy {
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
        };
        let lease_duration = ParticipantLeaseDuration::from(Duration::new(10, 11));

        let expected = SpdpDiscoveredParticipantData {
            dds_participant_data,
            participant_proxy,
            lease_duration,
        };

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
        let locator1 = Locator::new(11, 12, [1; 16]);
        let locator2 = Locator::new(21, 22, [2; 16]);

        let domain_id = 1;
        let domain_tag = "ab".to_string();
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid_prefix = GuidPrefix::from([8; 12]);
        let guid = Guid::new(guid_prefix, EntityId::new([0, 0, 1], EntityKind::BuiltInParticipant));
        let vendor_id = [73, 74];
        let expects_inline_qos = true;
        let metatraffic_unicast_locator_list = vec![locator1, locator2];
        let metatraffic_multicast_locator_list = vec![locator1];
        let default_unicast_locator_list = vec![locator1];
        let default_multicast_locator_list = vec![locator1];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );

        let dds_participant_data = ParticipantBuiltinTopicData {
            key: BuiltInTopicKey { value: guid.into() },
            user_data: UserDataQosPolicy { value: vec![] },
        };
        let participant_proxy = ParticipantProxy {
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
        };
        let lease_duration = ParticipantLeaseDuration::from(Duration::new(10, 11));

        let data = SpdpDiscoveredParticipantData {
            dds_participant_data,
            participant_proxy,
            lease_duration,
        };

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
