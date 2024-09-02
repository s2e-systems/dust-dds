use dust_dds_derive::XTypesSerialize;
use xtypes::serializer::SerializeMutableStruct;

use crate::{
    builtin_topics::ParticipantBuiltinTopicData,
    data_representation_builtin_endpoints::parameter_id_values::PID_USER_DATA,
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
pub(crate) struct DomainTag(String);
impl Default for DomainTag {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG.to_string())
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub(crate) struct DomainIdParameter(Option<DomainId>);
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
    pub(crate) domain_id: DomainIdParameter,
    #[parameter(id=PID_DOMAIN_TAG, default=Default::default())]
    pub(crate) domain_tag: DomainTag,
    #[parameter(id=PID_PROTOCOL_VERSION)]
    pub(crate) protocol_version: ProtocolVersion,
    #[parameter(id=PID_PARTICIPANT_GUID, skip_serialize)]
    pub(crate) guid_prefix: GuidPrefix,
    #[parameter(id=PID_VENDORID)]
    pub(crate) vendor_id: VendorId,
    #[parameter(id=PID_EXPECTS_INLINE_QOS, default=DEFAULT_EXPECTS_INLINE_QOS)]
    pub(crate) expects_inline_qos: bool,
    #[parameter(id=PID_METATRAFFIC_UNICAST_LOCATOR, collection)]
    pub(crate) metatraffic_unicast_locator_list: Vec<Locator>,
    #[parameter(id=PID_METATRAFFIC_MULTICAST_LOCATOR, collection)]
    pub(crate) metatraffic_multicast_locator_list: Vec<Locator>,
    #[parameter(id=PID_DEFAULT_UNICAST_LOCATOR, collection)]
    pub(crate) default_unicast_locator_list: Vec<Locator>,
    #[parameter(id=PID_DEFAULT_MULTICAST_LOCATOR, collection)]
    pub(crate) default_multicast_locator_list: Vec<Locator>,
    #[parameter(id=PID_BUILTIN_ENDPOINT_SET)]
    pub(crate) available_builtin_endpoints: BuiltinEndpointSet,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    #[parameter(id=PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, default=Default::default())]
    pub(crate) manual_liveliness_count: Count,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    #[parameter(id=PID_BUILTIN_ENDPOINT_QOS, default=Default::default())]
    pub(crate) builtin_endpoint_qos: BuiltinEndpointQos,
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
impl xtypes::serialize::XTypesSerialize for SpdpDiscoveredParticipantData {
    fn serialize(
        &self,
        serializer: impl xtypes::serialize::XTypesSerializer,
    ) -> Result<(), xtypes::error::XcdrError> {
        let mut p = serializer.serialize_mutable_struct()?;
        p.serialize_field(
            &self.dds_participant_data.key,
            PID_PARTICIPANT_GUID as u16,
            "key",
        )?;
        p.serialize_field(
            &self.dds_participant_data.user_data,
            PID_USER_DATA as u16,
            "user_data",
        )?;

        if let Some(domain_id) = &self.participant_proxy.domain_id.0 {
            p.serialize_field(domain_id, PID_DOMAIN_ID as u16, "domain_id")?;
        }

        //     #[parameter(id=PID_DOMAIN_TAG, default=Default::default())]
        //     pub(crate) domain_tag: DomainTag,
        p.serialize_field(
            &self.participant_proxy.domain_tag.0.as_str(),
            PID_DOMAIN_TAG as u16,
            "domain_tag",
        )?;

        //     #[parameter(id=PID_PROTOCOL_VERSION)]
        //     pub(crate) protocol_version: ProtocolVersion,
        p.serialize_field(
            &self.participant_proxy.protocol_version,
            PID_PROTOCOL_VERSION as u16,
            "protocol_version",
        )?;

        //     #[parameter(id=PID_PARTICIPANT_GUID, skip_serialize)]
        //     pub(crate) guid_prefix: GuidPrefix,
        p.serialize_field(
            &self.participant_proxy.guid_prefix,
            PID_PARTICIPANT_GUID as u16,
            "guid_prefix",
        )?;

        //     #[parameter(id=PID_VENDORID)]
        //     pub(crate) vendor_id: VendorId,
        p.serialize_field(
            &self.participant_proxy.vendor_id,
            PID_VENDORID as u16,
            "vendor_id",
        )?;

        //     #[parameter(id=PID_EXPECTS_INLINE_QOS, default=DEFAULT_EXPECTS_INLINE_QOS)]
        //     pub(crate) expects_inline_qos: bool,
        p.serialize_field(
            &self.participant_proxy.expects_inline_qos,
            PID_EXPECTS_INLINE_QOS as u16,
            "expects_inline_qos",
        )?;

        //     #[parameter(id=PID_METATRAFFIC_UNICAST_LOCATOR, collection)]
        //     pub(crate) metatraffic_unicast_locator_list: Vec<Locator>,
        for metatraffic_unicast_locator in &self.participant_proxy.metatraffic_unicast_locator_list
        {
            p.serialize_field(
                metatraffic_unicast_locator,
                PID_METATRAFFIC_UNICAST_LOCATOR as u16,
                "metatraffic_unicast_locator_list",
            )?;
        }

        //     #[parameter(id=PID_METATRAFFIC_MULTICAST_LOCATOR, collection)]
        //     pub(crate) metatraffic_multicast_locator_list: Vec<Locator>,
        for metatraffic_multicast_locator in
            &self.participant_proxy.metatraffic_multicast_locator_list
        {
            p.serialize_field(
                metatraffic_multicast_locator,
                PID_METATRAFFIC_MULTICAST_LOCATOR as u16,
                "metatraffic_multicast_locator_list",
            )?;
        }
        //     #[parameter(id=PID_DEFAULT_UNICAST_LOCATOR, collection)]
        //     pub(crate) default_unicast_locator_list: Vec<Locator>,
        for default_unicast_locator in &self.participant_proxy.default_unicast_locator_list {
            p.serialize_field(
                default_unicast_locator,
                PID_DEFAULT_UNICAST_LOCATOR as u16,
                "default_unicast_locator_list",
            )?;
        }
        //     #[parameter(id=PID_DEFAULT_MULTICAST_LOCATOR, collection)]
        //     pub(crate) default_multicast_locator_list: Vec<Locator>,
        for default_multicast_locator_list in &self.participant_proxy.default_multicast_locator_list
        {
            p.serialize_field(
                default_multicast_locator_list,
                PID_DEFAULT_MULTICAST_LOCATOR as u16,
                "default_multicast_locator_list",
            )?;
        }

        //     #[parameter(id=PID_BUILTIN_ENDPOINT_SET)]
        //     pub(crate) available_builtin_endpoints: BuiltinEndpointSet,
        p.serialize_field(
            &self.participant_proxy.available_builtin_endpoints,
            PID_BUILTIN_ENDPOINT_SET as u16,
            "available_builtin_endpoints",
        )?;

        //     // Default value is a deviation from the standard and is used for interoperability reasons:
        //     #[parameter(id=PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, default=Default::default())]
        //     pub(crate) manual_liveliness_count: Count,
        p.serialize_field(
            &self.participant_proxy.manual_liveliness_count,
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT as u16,
            "manual_liveliness_count",
        )?;

        //     // Default value is a deviation from the standard and is used for interoperability reasons:
        //     #[parameter(id=PID_BUILTIN_ENDPOINT_QOS, default=Default::default())]
        //     pub(crate) builtin_endpoint_qos: BuiltinEndpointQos,
        p.serialize_field(
            &self.participant_proxy.builtin_endpoint_qos,
            PID_BUILTIN_ENDPOINT_QOS as u16,
            "builtin_endpoint_qos",
        )?;

        // Default (DEFAULT_PARTICIPANT_LEASE_DURATION) is ommited compared to the standard due to interoperability reasons
        // #[parameter(id = PID_PARTICIPANT_LEASE_DURATION)]
        // lease_duration: Duration,
        p.serialize_field(
            &self.lease_duration,
            PID_PARTICIPANT_LEASE_DURATION as u16,
            "lease_duration",
        )?;

        // #[parameter(id = PID_DISCOVERED_PARTICIPANT, collection)]
        // discovered_participant_list: Vec<InstanceHandle>,
        for discovered_participant in &self.discovered_participant_list {
            p.serialize_field(
                discovered_participant,
                PID_DISCOVERED_PARTICIPANT as u16,
                "discovered_participant",
            )?;
        }

        p.end()
    }
}
impl<'de> xtypes::deserialize::XTypesDeserialize<'de> for SpdpDiscoveredParticipantData {
    fn deserialize(
        deserializer: impl xtypes::deserializer::XTypesDeserializer<'de>,
    ) -> Result<Self, xtypes::error::XcdrError> {
        todo!()
    }
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
    use xtypes::{
        error::XcdrError, serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer,
    };

    fn serialize_v1_le<T: XTypesSerialize, const N: usize>(v: &T) -> Result<[u8; N], XcdrError> {
        let mut buffer = [0; N];
        v.serialize(&mut Xcdr1LeSerializer::new(&mut buffer))?;
        Ok(buffer)
    }

    #[test]
    fn xtypes_serialize_spdp_discovered_participant_data() {
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

        let expected = Ok([
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
        ]);
        assert_eq!(serialize_v1_le(&data), expected);
    }

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
