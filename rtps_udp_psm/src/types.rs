#[derive(Clone, Copy)]
pub struct GuidPrefix(pub [u8; 12]);

impl rust_rtps_pim::types::GuidPrefix for GuidPrefix {
    const GUIDPREFIX_UNKNOWN: Self = Self([0; 12]);
}

impl Into<[u8; 12]> for GuidPrefix {
    fn into(self) -> [u8; 12] {
        self.0
    }
}

impl From<[u8; 12]> for GuidPrefix {
    fn from(value: [u8; 12]) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy)]
pub struct EntityId {
    pub entity_key: [u8; 3],
    pub entity_kind: u8,
}

impl EntityId {
    pub const ENTITY_KIND_USER_DEFINED_UNKNOWN: u8 = 0x00;
    pub const ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
    pub const ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
    pub const ENTITY_KIND_USER_DEFINED_READER_WITH_KEY: u8 = 0x04;
    pub const ENTITY_KIND_USER_DEFINED_READER_NO_KEY: u8 = 0x07;
    pub const ENTITY_KIND_USER_DEFINED_WRITER_GROUP: u8 = 0x08;
    pub const ENTITY_KIND_USER_DEFINED_READER_GROUP: u8 = 0x09;
    pub const ENTITY_KIND_BUILT_IN_UNKNOWN: u8 = 0xc0;
    pub const ENTITY_KIND_BUILT_IN_PARTICIPANT: u8 = 0xc1;
    pub const ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY: u8 = 0xc2;
    pub const ENTITY_KIND_BUILT_IN_WRITER_NO_KEY: u8 = 0xc3;
    pub const ENTITY_KIND_BUILT_IN_READER_WITH_KEY: u8 = 0xc4;
    pub const ENTITY_KIND_BUILT_IN_READER_NO_KEY: u8 = 0xc7;
    pub const ENTITY_KIND_BUILT_IN_WRITER_GROUP: u8 = 0xc8;
    pub const ENTITY_KIND_BUILT_IN_READER_GROUP: u8 = 0xc9;

    pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
        entity_key: [0, 0, 0x01],
        entity_kind: 0xc1,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x02],
        entity_kind: 0xc2,
    };
    pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x02],
        entity_kind: 0xc7,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x03],
        entity_kind: 0xc2,
    };
    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x03],
        entity_kind: 0xc7,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x04],
        entity_kind: 0xc2,
    };
    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x04],
        entity_kind: 0xc7,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0x01, 0x00],
        entity_kind: 0xc2,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0x01, 0x00],
        entity_kind: 0xc7,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
        entity_key: [0, 0x02, 0x00],
        entity_kind: 0xc2,
    };
    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
        entity_key: [0, 0x02, 0x00],
        entity_kind: 0xc7,
    };
}

impl rust_rtps_pim::types::EntityId for EntityId {
    const ENTITYID_UNKNOWN: Self = Self {
        entity_key: [0; 3],
        entity_kind: 0,
    };
}

impl Into<[u8; 4]> for EntityId {
    fn into(self) -> [u8; 4] {
        [
            self.entity_key[0],
            self.entity_key[1],
            self.entity_key[2],
            self.entity_kind,
        ]
    }
}

impl From<[u8; 4]> for EntityId {
    fn from(value: [u8; 4]) -> Self {
        Self {
            entity_key: [value[0], value[1], value[2]],
            entity_kind: value[3],
        }
    }
}

#[derive(Clone, Copy)]
pub struct GUID {
    pub guid_prefix: GuidPrefix,
    pub entity_id: EntityId,
}

impl rust_rtps_pim::types::GUID for GUID {
    const GUID_UNKNOWN: Self = Self {
        guid_prefix: <GuidPrefix as rust_rtps_pim::types::GuidPrefix>::GUIDPREFIX_UNKNOWN,
        entity_id: <EntityId as rust_rtps_pim::types::EntityId>::ENTITYID_UNKNOWN,
    };
}

impl Into<[u8; 16]> for GUID {
    fn into(self) -> [u8; 16] {
        [
            self.guid_prefix.0[0],
            self.guid_prefix.0[1],
            self.guid_prefix.0[2],
            self.guid_prefix.0[3],
            self.guid_prefix.0[4],
            self.guid_prefix.0[5],
            self.guid_prefix.0[6],
            self.guid_prefix.0[7],
            self.guid_prefix.0[8],
            self.guid_prefix.0[9],
            self.guid_prefix.0[10],
            self.guid_prefix.0[11],
            self.entity_id.entity_key[0],
            self.entity_id.entity_key[1],
            self.entity_id.entity_key[2],
            self.entity_id.entity_kind,
        ]
    }
}

impl From<[u8; 16]> for GUID {
    fn from(value: [u8; 16]) -> Self {
        Self {
            guid_prefix: [
                value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7],
                value[8], value[9], value[10], value[11],
            ]
            .into(),
            entity_id: [value[12], value[13], value[14], value[15]].into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceNumber {
    pub high: i32,
    pub low: u32,
}

impl rust_rtps_pim::types::SequenceNumber for SequenceNumber {
    const SEQUENCE_NUMBER_UNKNOWN: Self = Self {
        high: core::i32::MIN,
        low: core::u32::MAX,
    };
}

impl Into<i64> for SequenceNumber {
    fn into(self) -> i64 {
        ((self.high as i64) << 32) + self.low as i64
    }
}

impl From<i64> for SequenceNumber {
    fn from(value: i64) -> Self {
        Self {
            high: (value >> 32) as i32,
            low: value as u32,
        }
    }
}

pub struct Locator {
    pub kind: <Self as rust_rtps_pim::types::Locator>::Kind,
    pub port: <Self as rust_rtps_pim::types::Locator>::Port,
    pub address: <Self as rust_rtps_pim::types::Locator>::Address,
}
impl rust_rtps_pim::types::Locator for Locator {
    type Kind = i32;
    type Port = u32;
    type Address = [u8; 16];

    const LOCATOR_INVALID: Self = Self {
        kind: Self::LOCATOR_KIND_INVALID,
        port: Self::LOCATOR_PORT_INVALID,
        address: Self::LOCATOR_ADDRESS_INVALID,
    };

    const LOCATOR_KIND_INVALID: Self::Kind = -1;
    const LOCATOR_KIND_RESERVED: Self::Kind = 0;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::Kind = 1;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::Kind = 2;
    const LOCATOR_ADDRESS_INVALID: Self::Address = [0; 16];
    const LOCATOR_PORT_INVALID: Self::Port = 0;
}

#[derive(Clone, Copy)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl rust_rtps_pim::types::ProtocolVersion for ProtocolVersion {
    const PROTOCOLVERSION: Self = Self::PROTOCOLVERSION_2_4;
    const PROTOCOLVERSION_1_0: Self = Self { major: 1, minor: 0 };
    const PROTOCOLVERSION_1_1: Self = Self { major: 1, minor: 1 };
    const PROTOCOLVERSION_2_0: Self = Self { major: 2, minor: 0 };
    const PROTOCOLVERSION_2_1: Self = Self { major: 2, minor: 1 };
    const PROTOCOLVERSION_2_2: Self = Self { major: 2, minor: 2 };
    const PROTOCOLVERSION_2_3: Self = Self { major: 2, minor: 3 };
    const PROTOCOLVERSION_2_4: Self = Self { major: 2, minor: 4 };
}

#[derive(Clone, Copy)]
pub struct VendorId(pub [u8; 2]);

impl rust_rtps_pim::types::VendorId for VendorId {
    const VENDOR_ID_UNKNOWN: Self = Self([0; 2]);
}
