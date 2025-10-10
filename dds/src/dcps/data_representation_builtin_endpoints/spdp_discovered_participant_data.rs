use super::{
    parameter_id_values::{
        DEFAULT_EXPECTS_INLINE_QOS, DEFAULT_PARTICIPANT_LEASE_DURATION, PID_BUILTIN_ENDPOINT_QOS,
        PID_BUILTIN_ENDPOINT_SET, PID_DATA_REPRESENTATION, PID_DEADLINE,
        PID_DEFAULT_MULTICAST_LOCATOR, PID_DEFAULT_UNICAST_LOCATOR, PID_DESTINATION_ORDER,
        PID_DISCOVERED_PARTICIPANT, PID_DOMAIN_ID, PID_DOMAIN_TAG, PID_DURABILITY,
        PID_ENDPOINT_GUID, PID_EXPECTS_INLINE_QOS, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN,
        PID_LIVELINESS, PID_METATRAFFIC_MULTICAST_LOCATOR, PID_METATRAFFIC_UNICAST_LOCATOR,
        PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_PARTICIPANT_LEASE_DURATION,
        PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION, PID_RELIABILITY,
        PID_RESOURCE_LIMITS, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TRANSPORT_PRIORITY, PID_TYPE_NAME,
        PID_USER_DATA, PID_VENDORID,
    },
    payload_serializer_deserializer::parameter_list_deserializer::ParameterListCdrDeserializer,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    dcps::data_representation_builtin_endpoints::parameter_id_values::DEFAULT_DOMAIN_TAG,
    infrastructure::{
        domain::DomainId,
        error::DdsResult,
        instance::InstanceHandle,
        qos_policy::{UserDataQosPolicy, DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS},
        time::Duration,
        type_support::{DdsDeserialize, TypeSupport},
    },
    transport::types::{GuidPrefix, Locator, Long, ProtocolVersion, VendorId},
    xtypes::{
        binding::{DataKind, XTypesBinding},
        deserialize::XTypesDeserialize,
        dynamic_type::DynamicTypeBuilder,
    },
};
use alloc::{string::String, vec::Vec};

pub type Count = Long;

#[derive(PartialEq, Eq, Debug, Clone, Copy, XTypesDeserialize, TypeSupport)]
#[dust_dds(extensibility = "final", nested)]
pub struct BuiltinEndpointSet(pub u32);

impl Default for BuiltinEndpointSet {
    fn default() -> Self {
        Self(
            Self::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
                | Self::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
                | Self::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER
                | Self::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR
                | Self::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER
                | Self::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR
                | Self::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER
                | Self::BUILTIN_ENDPOINT_TOPICS_DETECTOR,
        )
    }
}

impl BuiltinEndpointSet {
    pub const BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER: u32 = 1 << 0;
    pub const BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR: u32 = 1 << 1;
    pub const BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER: u32 = 1 << 2;
    pub const BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR: u32 = 1 << 3;
    pub const BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER: u32 = 1 << 4;
    pub const BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR: u32 = 1 << 5;

    /*
    The following have been deprecated in version 2.4 of the
    specification. These bits should not be used by versions of the
    protocol equal to or newer than the deprecated version unless
    they are used with the same meaning as in versions prior to the
    deprecated version.
    @position(6) DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_ANNOUNCER,
    @position(7) DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_DETECTOR,
    @position(8) DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_ANNOUNCER,
    @position(9) DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_DETECTOR,
    */

    pub const _BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER: u32 = 1 << 10;
    pub const _BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER: u32 = 1 << 11;

    /*
    Bits 12-15 have been reserved by the DDS-Xtypes 1.2 Specification
    and future revisions thereof.
    Bits 16-27 have been reserved by the DDS-Security 1.1 Specification
    and future revisions thereof.
    */

    pub const BUILTIN_ENDPOINT_TOPICS_ANNOUNCER: u32 = 1 << 28;
    pub const BUILTIN_ENDPOINT_TOPICS_DETECTOR: u32 = 1 << 29;

    #[allow(dead_code)]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn has(&self, endpoint: u32) -> bool {
        (self.0 & endpoint) == endpoint
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy, XTypesDeserialize, TypeSupport)]
#[dust_dds(extensibility = "final", nested)]
pub struct BuiltinEndpointQos(pub u32);

impl BuiltinEndpointQos {
    #[allow(dead_code)]
    pub const BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER: u32 = 1 << 29;

    #[allow(dead_code)]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    #[allow(dead_code)]
    pub fn has(&self, endpoint: u32) -> bool {
        (self.0 & endpoint) == endpoint
    }
}

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "mutable")]
pub struct ParticipantProxy {
    #[dust_dds(id = PID_DOMAIN_ID as u32, non_serialized)]
    pub(crate) domain_id: Option<DomainId>,
    #[dust_dds(id = PID_DOMAIN_TAG as u32, optional)]
    pub(crate) domain_tag: String,
    #[dust_dds(id = PID_PROTOCOL_VERSION as u32)]
    pub(crate) protocol_version: ProtocolVersion,
    #[dust_dds(id = PID_PARTICIPANT_GUID as u32, non_serialized)]
    pub(crate) guid_prefix: GuidPrefix,
    #[dust_dds(id = PID_VENDORID as u32)]
    pub(crate) vendor_id: VendorId,
    #[dust_dds(id = PID_EXPECTS_INLINE_QOS as u32, optional, default_value=DEFAULT_EXPECTS_INLINE_QOS)]
    pub(crate) expects_inline_qos: bool,
    #[dust_dds(id = PID_METATRAFFIC_UNICAST_LOCATOR as u32, optional)]
    pub(crate) metatraffic_unicast_locator_list: Vec<Locator>,
    #[dust_dds(id = PID_METATRAFFIC_MULTICAST_LOCATOR as u32, optional)]
    pub(crate) metatraffic_multicast_locator_list: Vec<Locator>,
    #[dust_dds(id = PID_DEFAULT_UNICAST_LOCATOR as u32, optional)]
    pub(crate) default_unicast_locator_list: Vec<Locator>,
    #[dust_dds(id = PID_DEFAULT_MULTICAST_LOCATOR as u32, optional)]
    pub(crate) default_multicast_locator_list: Vec<Locator>,
    #[dust_dds(id = PID_BUILTIN_ENDPOINT_SET as u32, optional)]
    pub(crate) available_builtin_endpoints: BuiltinEndpointSet,
    #[dust_dds(id = PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT as u32)]
    pub(crate) manual_liveliness_count: Count,
    #[dust_dds(id = PID_BUILTIN_ENDPOINT_QOS as u32)]
    pub(crate) builtin_endpoint_qos: BuiltinEndpointQos,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SpdpDiscoveredParticipantData {
    pub(crate) dds_participant_data: ParticipantBuiltinTopicData,
    pub(crate) participant_proxy: ParticipantProxy,
    pub(crate) lease_duration: Duration,
    pub(crate) discovered_participant_list: Vec<InstanceHandle>,
}

impl dust_dds::infrastructure::type_support::TypeSupport for SpdpDiscoveredParticipantData {
    fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
        extern crate alloc;
        struct ConvenienceDynamicTypeBuilder {
            builder: DynamicTypeBuilder,
            index: u32,
        }
        impl ConvenienceDynamicTypeBuilder {
            fn add_member<T: XTypesBinding>(&mut self, name: &str, id: i16) {
                self.builder
                    .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                        name: alloc::string::String::from(name),
                        id: id as u32,
                        r#type: T::get_dynamic_type(),
                        default_value: None,
                        index: self.index,
                        try_construct_kind:
                            dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                        label: alloc::vec::Vec::new(),
                        is_key: false,
                        is_optional: false,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    })
                    .unwrap();
                self.index += 1;
            }
            fn add_key_member<T: XTypesBinding>(&mut self, name: &str, id: i16) {
                self.builder
                    .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                        name: alloc::string::String::from(name),
                        id: id as u32,
                        r#type: T::get_dynamic_type(),
                        default_value: None,
                        index: self.index,
                        try_construct_kind:
                            dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                        label: alloc::vec::Vec::new(),
                        is_key: true,
                        is_optional: true,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    })
                    .unwrap();
                self.index += 1;
            }
            fn add_member_with_default<T: XTypesBinding + Into<DataKind>>(
                &mut self,
                name: &str,
                id: i16,
                default: T,
            ) {
                self.builder
                    .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                        name: alloc::string::String::from(name),
                        id: id as u32,
                        r#type: T::get_dynamic_type(),
                        default_value: Some(default.into()),
                        index: self.index,
                        try_construct_kind:
                            dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                        label: alloc::vec::Vec::new(),
                        is_key: false,
                        is_optional: true,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    })
                    .unwrap();
                self.index += 1;
            }
        }
        let mut builder = ConvenienceDynamicTypeBuilder {
            builder: dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
                dust_dds::xtypes::dynamic_type::TypeDescriptor {
                    kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
                    name: alloc::string::String::from("SpdpDiscoveredParticipantData"),
                    base_type: None,
                    discriminator_type: None,
                    bound: alloc::vec::Vec::new(),
                    element_type: None,
                    key_element_type: None,
                    extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Mutable,
                    is_nested: false,
                },
            ),
            index: 0,
        };
        builder.add_key_member::<BuiltInTopicKey>("key", PID_PARTICIPANT_GUID);
        builder.add_member_with_default("user_data", PID_USER_DATA, UserDataQosPolicy::default());
        builder.add_member::<DomainId>("domain_id", PID_DOMAIN_ID);
        builder.add_member_with_default(
            "domain_tag",
            PID_DOMAIN_TAG,
            String::from(DEFAULT_DOMAIN_TAG),
        );
        builder.add_member::<ProtocolVersion>("protocol_version", PID_PROTOCOL_VERSION);
        // builder.add_member::<GuidPrefix>("guid_prefix", PID_PARTICIPANT_GUID);
        builder.add_member::<VendorId>("vendor_id", PID_VENDORID);
        builder.add_member_with_default(
            "expects_inline_qos",
            PID_EXPECTS_INLINE_QOS,
            DEFAULT_EXPECTS_INLINE_QOS,
        );
        builder.add_member_with_default(
            "metatraffic_unicast_locator_list",
            PID_METATRAFFIC_UNICAST_LOCATOR,
            Vec::<Locator>::new(),
        );
        builder.add_member_with_default(
            "metatraffic_multicast_locator_list",
            PID_METATRAFFIC_MULTICAST_LOCATOR,
            Vec::<Locator>::new(),
        );
        builder.add_member_with_default(
            "default_unicast_locator_list",
            PID_DEFAULT_UNICAST_LOCATOR,
            Vec::<Locator>::new(),
        );
        builder.add_member_with_default(
            "default_multicast_locator_list",
            PID_DEFAULT_MULTICAST_LOCATOR,
            Vec::<Locator>::new(),
        );
        builder.add_member::<BuiltinEndpointSet>(
            "available_builtin_endpoints",
            PID_BUILTIN_ENDPOINT_SET,
        );
        builder.add_member_with_default(
            "manual_liveliness_count",
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            0,
        );
        builder.add_member_with_default(
            "builtin_endpoint_qos",
            PID_BUILTIN_ENDPOINT_QOS,
            BuiltinEndpointQos::default(),
        );
        // of interoperability reasons the lease_duration is made mandatory
        builder.add_member::<Duration>("lease_duration", PID_PARTICIPANT_LEASE_DURATION);
        builder.builder.build()
    }

    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        let data =
            dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type())
                .set_value(PID_PARTICIPANT_GUID as u32, self.dds_participant_data.key)
                .set_value(PID_USER_DATA as u32, self.dds_participant_data.user_data);
        if let Some(domain_id) = self.participant_proxy.domain_id {
            data.set_value(PID_DOMAIN_ID as u32, domain_id)
        } else {
            data
        }
        .set_value(PID_DOMAIN_TAG as u32, self.participant_proxy.domain_tag)
        .set_value(
            PID_PROTOCOL_VERSION as u32,
            self.participant_proxy.protocol_version,
        )
        // self.participant_proxy.guid_prefix is ommitted
        .set_value(PID_VENDORID as u32, self.participant_proxy.vendor_id)
        .set_value(
            PID_EXPECTS_INLINE_QOS as u32,
            self.participant_proxy.expects_inline_qos,
        )
        .set_value(
            PID_METATRAFFIC_UNICAST_LOCATOR as u32,
            self.participant_proxy.metatraffic_unicast_locator_list,
        )
        .set_value(
            PID_METATRAFFIC_MULTICAST_LOCATOR as u32,
            self.participant_proxy.metatraffic_multicast_locator_list,
        )
        .set_value(
            PID_DEFAULT_UNICAST_LOCATOR as u32,
            self.participant_proxy.default_unicast_locator_list,
        )
        .set_value(
            PID_DEFAULT_MULTICAST_LOCATOR as u32,
            self.participant_proxy.default_multicast_locator_list,
        )
        .set_value(
            PID_BUILTIN_ENDPOINT_SET as u32,
            self.participant_proxy.available_builtin_endpoints,
        )
        .set_value(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT as u32,
            self.participant_proxy.manual_liveliness_count,
        )
        .set_value(
            PID_BUILTIN_ENDPOINT_QOS as u32,
            self.participant_proxy.builtin_endpoint_qos,
        )
        .set_value(PID_PARTICIPANT_LEASE_DURATION as u32, self.lease_duration)
    }
}

impl<'de> DdsDeserialize<'de> for ParticipantBuiltinTopicData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;
        Ok(Self {
            key: pl_deserializer.read(PID_PARTICIPANT_GUID)?,
            user_data: pl_deserializer.read_with_default(PID_USER_DATA, Default::default())?,
        })
    }
}

impl<'de> DdsDeserialize<'de> for TopicBuiltinTopicData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;
        Ok(Self {
            key: pl_deserializer.read(PID_ENDPOINT_GUID)?,
            name: pl_deserializer.read(PID_TOPIC_NAME)?,
            type_name: pl_deserializer.read(PID_TYPE_NAME)?,
            durability: pl_deserializer.read_with_default(PID_DURABILITY, Default::default())?,
            deadline: pl_deserializer.read_with_default(PID_DEADLINE, Default::default())?,
            latency_budget: pl_deserializer
                .read_with_default(PID_LATENCY_BUDGET, Default::default())?,
            liveliness: pl_deserializer.read_with_default(PID_LIVELINESS, Default::default())?,
            reliability: pl_deserializer.read_with_default(
                PID_RELIABILITY,
                DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            )?,
            transport_priority: pl_deserializer
                .read_with_default(PID_TRANSPORT_PRIORITY, Default::default())?,
            lifespan: pl_deserializer.read_with_default(PID_LIFESPAN, Default::default())?,
            destination_order: pl_deserializer
                .read_with_default(PID_DESTINATION_ORDER, Default::default())?,
            history: pl_deserializer.read_with_default(PID_HISTORY, Default::default())?,
            resource_limits: pl_deserializer
                .read_with_default(PID_RESOURCE_LIMITS, Default::default())?,
            ownership: pl_deserializer.read_with_default(PID_OWNERSHIP, Default::default())?,
            topic_data: pl_deserializer.read_with_default(PID_TOPIC_DATA, Default::default())?,
            representation: pl_deserializer
                .read_with_default(PID_DATA_REPRESENTATION, Default::default())?,
        })
    }
}

impl<'de> DdsDeserialize<'de> for SpdpDiscoveredParticipantData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let pl_deserializer = ParameterListCdrDeserializer::new(serialized_data)?;
        let domain_id = pl_deserializer.read(PID_DOMAIN_ID).ok();
        Ok(Self {
            dds_participant_data: ParticipantBuiltinTopicData::deserialize_data(serialized_data)?,
            participant_proxy: ParticipantProxy {
                domain_id: domain_id.unwrap_or_default(),
                domain_tag: pl_deserializer
                    .read_with_default(PID_DOMAIN_TAG, Default::default())?,
                protocol_version: pl_deserializer.read(PID_PROTOCOL_VERSION)?,
                guid_prefix: pl_deserializer.read(PID_PARTICIPANT_GUID)?,
                vendor_id: pl_deserializer.read(PID_VENDORID)?,
                expects_inline_qos: pl_deserializer
                    .read_with_default(PID_EXPECTS_INLINE_QOS, DEFAULT_EXPECTS_INLINE_QOS)?,
                metatraffic_unicast_locator_list: pl_deserializer
                    .read_collection(PID_METATRAFFIC_UNICAST_LOCATOR)?,
                metatraffic_multicast_locator_list: pl_deserializer
                    .read_collection(PID_METATRAFFIC_MULTICAST_LOCATOR)?,
                default_unicast_locator_list: pl_deserializer
                    .read_collection(PID_DEFAULT_UNICAST_LOCATOR)?,
                default_multicast_locator_list: pl_deserializer
                    .read_collection(PID_DEFAULT_MULTICAST_LOCATOR)?,
                available_builtin_endpoints: pl_deserializer.read(PID_BUILTIN_ENDPOINT_SET)?,
                // Default value is a deviation from the standard and is used for interoperability reasons:
                manual_liveliness_count: pl_deserializer.read_with_default(
                    PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
                    Default::default(),
                )?,
                // Default value is a deviation from the standard and is used for interoperability reasons:
                builtin_endpoint_qos: pl_deserializer
                    .read_with_default(PID_BUILTIN_ENDPOINT_QOS, Default::default())?,
            },
            lease_duration: pl_deserializer.read_with_default(
                PID_PARTICIPANT_LEASE_DURATION,
                DEFAULT_PARTICIPANT_LEASE_DURATION,
            )?,
            discovered_participant_list: pl_deserializer
                .read_collection(PID_DISCOVERED_PARTICIPANT)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        infrastructure::qos_policy::UserDataQosPolicy,
        rtps::types::PROTOCOLVERSION_2_4,
        xtypes::{pl_cdr_serializer::PlCdrLeSerializer, serialize::XTypesSerialize},
    };

    #[test]
    fn serialize_spdp_discovered_participant_data() {
        let locator1 = Locator::new(11, 12, [1; 16]);
        let locator2 = Locator::new(21, 22, [2; 16]);

        let data = SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 1, 0xc1],
                },
                user_data: UserDataQosPolicy { value: vec![] },
            },
            participant_proxy: ParticipantProxy {
                domain_id: Some(0),
                domain_tag: "ab".to_string(),
                protocol_version: ProtocolVersion::new(2, 4),
                guid_prefix: [8; 12],
                vendor_id: [73, 74],
                expects_inline_qos: true,
                metatraffic_unicast_locator_list: vec![locator1, locator2],
                metatraffic_multicast_locator_list: vec![locator1],
                default_unicast_locator_list: vec![locator1],
                default_multicast_locator_list: vec![locator1],
                available_builtin_endpoints: BuiltinEndpointSet::new(
                    BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR,
                ),
                manual_liveliness_count: 2,
                builtin_endpoint_qos: BuiltinEndpointQos::new(
                    BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
                ),
            },
            lease_duration: Duration::new(10, 11),
            discovered_participant_list: vec![],
        };

        let expected = vec![
            // 0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x02, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION
            10, 0x00, 0x00, 0x00, // Duration: seconds
            11, 0x00, 0x00, 0x00, // Duration: fraction
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x00, 0x00, 0x00, 0x00, // DomainId
            0x15, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion
            0x16, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            0x31, 0x00, 24, 0x00, // PID_DEFAULT_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
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
            0x34, 0x00, 4, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT
            0x02, 0x00, 0x00, 0x00, // Count
            0x43, 0x00, 0x04, 0x00, // PID_EXPECTS_INLINE_QOS, Length: 4,
            0x01, 0x00, 0x00, 0x00, // True
            0x48, 0x00, 24, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x50, 0x00, 16, 0x00, // PID_PARTICIPANT_GUID, Length
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            0, 0, 1, 0xc1, // EntityId
            0x58, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET
            0x02, 0x00, 0x00, 0x00, //
            0x77, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_QOS
            0x00, 0x00, 0x00, 0x20, //
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        let dynamic_sample = data.create_dynamic_sample();

        let mut buffer = vec![];
        dynamic_sample
            .serialize(&mut PlCdrLeSerializer::new(&mut buffer))
            .unwrap();
        assert_eq!(buffer, expected);
    }

    #[test]
    fn serialize_spdp_discovered_participant_data_all_default() {
        let data = SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 1, 0xc1],
                },
                user_data: UserDataQosPolicy::default(),
            },
            participant_proxy: ParticipantProxy {
                domain_id: None,
                domain_tag: String::from(DEFAULT_DOMAIN_TAG),
                protocol_version: PROTOCOLVERSION_2_4,
                guid_prefix: GuidPrefix::default(),
                vendor_id: [73, 74],
                expects_inline_qos: DEFAULT_EXPECTS_INLINE_QOS,
                metatraffic_unicast_locator_list: Vec::new(),
                metatraffic_multicast_locator_list: Vec::new(),
                default_unicast_locator_list: Vec::new(),
                default_multicast_locator_list: Vec::new(),
                available_builtin_endpoints: BuiltinEndpointSet::new(
                    BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR,
                ),
                manual_liveliness_count: 0,
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            },
            lease_duration: DEFAULT_PARTICIPANT_LEASE_DURATION,
            discovered_participant_list: Vec::new(),
        };

        let expected = vec![
            // 0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x02, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION
            100, 0x00, 0x00, 0x00, // Duration: seconds
            0, 0x00, 0x00, 0x00, // Duration: fraction
            0x15, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion
            0x16, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            0x50, 0x00, 16, 0x00, // PID_PARTICIPANT_GUID, Length
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            0, 0, 1, 0xc1, // EntityId
            0x58, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET
            0x02, 0x00, 0x00, 0x00, //
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        let dynamic_sample = data.create_dynamic_sample();

        let mut buffer = vec![];
        dynamic_sample
            .serialize(&mut PlCdrLeSerializer::new(&mut buffer))
            .unwrap();
        assert_eq!(buffer, expected);
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

        let expected = SpdpDiscoveredParticipantData {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: [8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 1, 0xc1],
                },
                user_data: UserDataQosPolicy { value: vec![] },
            },
            participant_proxy: ParticipantProxy {
                domain_id: Some(domain_id),
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
            discovered_participant_list: vec![],
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
        let result = SpdpDiscoveredParticipantData::deserialize_data(&mut data).unwrap();
        assert_eq!(result, expected);
    }
}
