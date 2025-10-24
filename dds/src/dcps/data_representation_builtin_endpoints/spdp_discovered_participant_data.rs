use super::parameter_id_values::{
    DEFAULT_EXPECTS_INLINE_QOS, PID_BUILTIN_ENDPOINT_QOS, PID_BUILTIN_ENDPOINT_SET,
    PID_DEFAULT_MULTICAST_LOCATOR, PID_DEFAULT_UNICAST_LOCATOR, PID_DOMAIN_ID, PID_DOMAIN_TAG,
    PID_EXPECTS_INLINE_QOS, PID_METATRAFFIC_MULTICAST_LOCATOR, PID_METATRAFFIC_UNICAST_LOCATOR,
    PID_PARTICIPANT_GUID, PID_PARTICIPANT_LEASE_DURATION, PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
    PID_PROTOCOL_VERSION, PID_USER_DATA, PID_VENDORID,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, ParticipantBuiltinTopicData},
    dcps::data_representation_builtin_endpoints::parameter_id_values::DEFAULT_DOMAIN_TAG,
    infrastructure::{
        domain::DomainId, instance::InstanceHandle, qos_policy::UserDataQosPolicy, time::Duration,
        type_support::TypeSupport,
    },
    transport::types::{GuidPrefix, Locator, Long, ProtocolVersion, VendorId},
    xtypes::{
        binding::XTypesBinding, data_storage::DataStorageMapping, dynamic_type::DynamicTypeBuilder,
    },
};
use alloc::{string::String, vec, vec::Vec};

pub type Count = Long;

#[derive(PartialEq, Eq, Debug, Clone, Copy, TypeSupport)]
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

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy, TypeSupport)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParticipantProxy {
    pub(crate) domain_id: Option<DomainId>,
    pub(crate) domain_tag: String,
    pub(crate) protocol_version: ProtocolVersion,
    pub(crate) guid_prefix: GuidPrefix,
    pub(crate) vendor_id: VendorId,
    pub(crate) expects_inline_qos: bool,
    pub(crate) metatraffic_unicast_locator_list: Vec<Locator>,
    pub(crate) metatraffic_multicast_locator_list: Vec<Locator>,
    pub(crate) default_unicast_locator_list: Vec<Locator>,
    pub(crate) default_multicast_locator_list: Vec<Locator>,
    pub(crate) available_builtin_endpoints: BuiltinEndpointSet,
    pub(crate) manual_liveliness_count: Count,
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
                        is_optional: false,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    })
                    .unwrap();
                self.index += 1;
            }
            fn add_member_with_default<T: XTypesBinding + DataStorageMapping>(
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
                        default_value: Some(default.into_storage()),
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
        builder.add_member_with_default::<DomainId>("domain_id", PID_DOMAIN_ID, -1);
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

    fn create_sample(src: crate::xtypes::dynamic_type::DynamicData) -> Self {
        Self {
            dds_participant_data: ParticipantBuiltinTopicData {
                key: DataStorageMapping::try_from_storage(
                    src.get_value(PID_PARTICIPANT_GUID as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                user_data: DataStorageMapping::try_from_storage(
                    src.get_value(PID_USER_DATA as u32).expect("Must exist"),
                )
                .expect("Type must match"),
            },
            participant_proxy: ParticipantProxy {
                domain_id: None,
                domain_tag: DataStorageMapping::try_from_storage(
                    src.get_value(PID_DOMAIN_TAG as u32).expect("Must exist"),
                )
                .expect("Type must match"),
                protocol_version: DataStorageMapping::try_from_storage(
                    src.get_value(PID_PROTOCOL_VERSION as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                guid_prefix: <BuiltInTopicKey>::try_from_storage(
                    src.get_value(PID_PARTICIPANT_GUID as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match")
                .value[0..12]
                    .try_into()
                    .expect("Must match"),
                vendor_id: DataStorageMapping::try_from_storage(
                    src.get_value(PID_VENDORID as u32).expect("Must exist"),
                )
                .expect("Type must match"),
                expects_inline_qos: DataStorageMapping::try_from_storage(
                    src.get_value(PID_EXPECTS_INLINE_QOS as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                metatraffic_unicast_locator_list: DataStorageMapping::try_from_storage(
                    src.get_value(PID_METATRAFFIC_UNICAST_LOCATOR as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                metatraffic_multicast_locator_list: DataStorageMapping::try_from_storage(
                    src.get_value(PID_METATRAFFIC_MULTICAST_LOCATOR as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                default_unicast_locator_list: DataStorageMapping::try_from_storage(
                    src.get_value(PID_DEFAULT_UNICAST_LOCATOR as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                default_multicast_locator_list: DataStorageMapping::try_from_storage(
                    src.get_value(PID_DEFAULT_MULTICAST_LOCATOR as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                available_builtin_endpoints: DataStorageMapping::try_from_storage(
                    src.get_value(PID_BUILTIN_ENDPOINT_SET as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                manual_liveliness_count: DataStorageMapping::try_from_storage(
                    src.get_value(PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
                builtin_endpoint_qos: DataStorageMapping::try_from_storage(
                    src.get_value(PID_BUILTIN_ENDPOINT_QOS as u32)
                        .expect("Must exist"),
                )
                .expect("Type must match"),
            },
            lease_duration: DataStorageMapping::try_from_storage(
                src.get_value(PID_PARTICIPANT_LEASE_DURATION as u32)
                    .expect("Must exist"),
            )
            .expect("Type must match"),
            discovered_participant_list: vec![], // DataStorageMapping::try_from_storage(
                                                 //     src.get_value(PID_DISCOVERED_PARTICIPANT as u32)
                                                 //         .expect("Must exist"),
                                                 // )
                                                 // .expect("Type must match"),
        }
    }

    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        let mut data =
            dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(Self::get_type());
        data.set_value(
            PID_PARTICIPANT_GUID as u32,
            self.dds_participant_data.key.into_storage(),
        );
        data.set_value(
            PID_USER_DATA as u32,
            self.dds_participant_data.user_data.into_storage(),
        );
        if let Some(domain_id) = self.participant_proxy.domain_id {
            data.set_value(PID_DOMAIN_ID as u32, domain_id.into_storage());
        }
        data.set_value(
            PID_DOMAIN_TAG as u32,
            self.participant_proxy.domain_tag.into_storage(),
        );
        data.set_value(
            PID_PROTOCOL_VERSION as u32,
            self.participant_proxy.protocol_version.into_storage(),
        );
        // self.participant_proxy.guid_prefix is ommitted
        data.set_value(
            PID_VENDORID as u32,
            self.participant_proxy.vendor_id.into_storage(),
        );
        data.set_value(
            PID_EXPECTS_INLINE_QOS as u32,
            self.participant_proxy.expects_inline_qos.into_storage(),
        );
        data.set_value(
            PID_METATRAFFIC_UNICAST_LOCATOR as u32,
            self.participant_proxy
                .metatraffic_unicast_locator_list
                .into_storage(),
        );
        data.set_value(
            PID_METATRAFFIC_MULTICAST_LOCATOR as u32,
            self.participant_proxy
                .metatraffic_multicast_locator_list
                .into_storage(),
        );
        data.set_value(
            PID_DEFAULT_UNICAST_LOCATOR as u32,
            self.participant_proxy
                .default_unicast_locator_list
                .into_storage(),
        );
        data.set_value(
            PID_DEFAULT_MULTICAST_LOCATOR as u32,
            self.participant_proxy
                .default_multicast_locator_list
                .into_storage(),
        );
        data.set_value(
            PID_BUILTIN_ENDPOINT_SET as u32,
            self.participant_proxy
                .available_builtin_endpoints
                .into_storage(),
        );
        data.set_value(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT as u32,
            self.participant_proxy
                .manual_liveliness_count
                .into_storage(),
        );
        data.set_value(
            PID_BUILTIN_ENDPOINT_QOS as u32,
            self.participant_proxy.builtin_endpoint_qos.into_storage(),
        );
        data.set_value(
            PID_PARTICIPANT_LEASE_DURATION as u32,
            self.lease_duration.into_storage(),
        );
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtin_topics::BuiltInTopicKey,
        dcps::data_representation_builtin_endpoints::parameter_id_values::DEFAULT_PARTICIPANT_LEASE_DURATION,
        infrastructure::qos_policy::UserDataQosPolicy,
        rtps::types::PROTOCOLVERSION_2_4,
        xtypes::{
            data_representation::{cdr_reader::PlCdr1Deserializer, endianness::LittleEndian},
            dynamic_type::DynamicData,
            serializer::RtpsPlCdrSerializer,
        },
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
                user_data: UserDataQosPolicy {
                    value: vec![97, 53],
                },
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
        }
        .create_dynamic_sample();

        let expected = [
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            2, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION
            10, 0x00, 0x00, 0x00, // Duration: seconds
            11, 0x00, 0x00, 0x00, // Duration: fraction
            15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x00, 0x00, 0x00, 0x00, // DomainId
            21, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion
            22, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            44, 0x00, 8, 0x00, // PID_USER_DATA, Length
            2, 0, 0, 0, // sequence length
            97, 53, 0, 0, // data, padding (2 bytes)
            49, 0x00, 24, 0x00, // PID_DEFAULT_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            50, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            50, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR
            21, 0x00, 0x00, 0x00, // Locator{kind
            22, 0x00, 0x00, 0x00, // port,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            51, 0x00, 24, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            52, 0x00, 4, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT
            0x02, 0x00, 0x00, 0x00, // Count
            0x43, 0x00, 0x04, 0x00, // PID_EXPECTS_INLINE_QOS, Length: 4,
            0x01, 0x00, 0x00, 0x00, // True
            72, 0x00, 24, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR
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
            88, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET
            0x02, 0x00, 0x00, 0x00, //
            119, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_QOS
            0x00, 0x00, 0x00, 0x20, //
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        assert_eq!(
            RtpsPlCdrSerializer::serialize(Vec::new(), &data).unwrap(),
            expected
        );
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
        }
        .create_dynamic_sample();

        let expected = [
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
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
            88, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET
            0x02, 0x00, 0x00, 0x00, //
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];
        assert_eq!(
            RtpsPlCdrSerializer::serialize(Vec::new(), &data).unwrap(),
            expected
        );
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
        }
        .create_dynamic_sample();

        let data = [
            15, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            3, 0x00, 0x00, 0x00, // DomainTag: string length (incl. terminator)
            b'a', b'b', 0, 0x00, // DomainTag: string + padding (1 byte)
            21, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion
            80, 0x00, 16, 0x00, // PID_PARTICIPANT_GUID, Length
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            0, 0, 1, 0xc1, // EntityId,
            22, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            67, 0x00, 0x04, 0x00, // PID_EXPECTS_INLINE_QOS, Length: 4,
            0x01, 0x00, 0x00, 0x00, // True
            50, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            50, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR
            21, 0x00, 0x00, 0x00, // Locator{kind
            22, 0x00, 0x00, 0x00, // port,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            51, 0x00, 24, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            49, 0x00, 24, 0x00, // PID_DEFAULT_UNICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            72, 0x00, 24, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR
            11, 0x00, 0x00, 0x00, // Locator{kind
            12, 0x00, 0x00, 0x00, // port,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            88, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET
            0x02, 0x00, 0x00, 0x00, //
            0x34, 0x00, 4, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT
            0x02, 0x00, 0x00, 0x00, // Count
            119, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_QOS
            0x00, 0x00, 0x00, 0x20, //
            2, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION
            10, 0x00, 0x00, 0x00, // Duration: seconds
            11, 0x00, 0x00, 0x00, // Duration: fraction
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ];

        assert_eq!(
            DynamicData::xcdr_deserialize(
                SpdpDiscoveredParticipantData::get_type(),
                &mut PlCdr1Deserializer::new(&data, LittleEndian)
            )
            .unwrap(),
            expected
        );
    }
}
