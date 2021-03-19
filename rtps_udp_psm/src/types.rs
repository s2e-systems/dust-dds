use std::ops::Deref;

pub struct UShort(pub u16);

impl Into<[u8; 2]> for UShort {
    fn into(self) -> [u8; 2] {
        self.0.to_ne_bytes()
    }
}

impl Deref for UShort {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Short(pub i16);

impl Into<[u8; 2]> for Short {
    fn into(self) -> [u8; 2] {
        self.0.to_ne_bytes()
    }
}

impl Deref for Short {
    type Target = i16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ULong(pub u32);

impl Into<[u8; 4]> for ULong {
    fn into(self) -> [u8; 4] {
        self.0.to_ne_bytes()
    }
}

impl Deref for ULong {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Long(pub i32);

impl Into<[u8; 4]> for Long {
    fn into(self) -> [u8; 4] {
        self.0.to_ne_bytes()
    }
}

impl Deref for Long {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct GuidPrefix(pub [u8; 12]);

impl rust_rtps_pim::types::GuidPrefix for GuidPrefix {
    const GUID_PREFIX_UNKNOWN: Self = Self([0; 12]);
}

impl Into<[u8; 12]> for GuidPrefix {
    fn into(self) -> [u8; 12] {
        self.0
    }
}

pub struct EntityId {
    entity_key: [u8; 3],
    entity_kind: u8,
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

pub struct GUID {
    guid_prefix: GuidPrefix,
    entity_id: EntityId,
}

impl rust_rtps_pim::types::GUID for GUID {
    const GUID_UNKNOWN: Self = Self {
        guid_prefix: <GuidPrefix as rust_rtps_pim::types::GuidPrefix>::GUID_PREFIX_UNKNOWN,
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

pub struct SequenceNumber {
    high: Long,
    low: ULong,
}

impl rust_rtps_pim::types::SequenceNumber for SequenceNumber {
    const SEQUENCE_NUMBER_UNKNOWN: Self = Self {
        high: Long(core::i32::MIN),
        low: ULong(core::u32::MIN),
    };
}

impl Into<i64> for SequenceNumber {
    fn into(self) -> i64 {
        ((*self.high as i64) << 32) + *self.low as i64
    }
}

pub struct Locator {
    kind: Long,
    port: ULong,
    address: [u8; 16],
}

impl rust_rtps_pim::types::Locator for Locator {
    type Kind = Long;

    type Port = ULong;

    type Address = [u8; 16];

    const LOCATOR_INVALID: Self = Self {
        kind: Self::LOCATOR_KIND_INVALID,
        port: Self::LOCATOR_PORT_INVALID,
        address: Self::LOCATOR_ADDRESS_INVALID,
    };

    const LOCATOR_KIND_INVALID: Self::Kind = Long(-1);

    const LOCATOR_KIND_RESERVED: Self::Kind = Long(0);
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::Kind = Long(1);
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::Kind = Long(2);

    const LOCATOR_ADDRESS_INVALID: Self::Address = [0; 16];

    const LOCATOR_PORT_INVALID: Self::Port = ULong(0);

    fn kind(&self) -> &Self::Kind {
        &self.kind
    }

    fn port(&self) -> &Self::Port {
        &self.port
    }

    fn address(&self) -> &Self::Address {
        &self.address
    }
}

pub struct ProtocolVersion {
    major: u8,
    minor: u8,
}

impl rust_rtps_pim::types::ProtocolVersion for ProtocolVersion {
    const PROTOCOL_VERSION: Self = Self::PROTOCOL_VERSION_2_4;

    const PROTOCOL_VERSION_1_0: Self = Self { major: 1, minor: 0 };

    const PROTOCOL_VERSION_1_1: Self = Self { major: 1, minor: 1 };

    const PROTOCOL_VERSION_2_0: Self = Self { major: 2, minor: 0 };

    const PROTOCOL_VERSION_2_1: Self = Self { major: 2, minor: 1 };

    const PROTOCOL_VERSION_2_2: Self = Self { major: 2, minor: 2 };

    const PROTOCOL_VERSION_2_3: Self = Self { major: 2, minor: 3 };

    const PROTOCOL_VERSION_2_4: Self = Self { major: 2, minor: 4 };
}

pub struct VendorId([u8; 2]);

impl rust_rtps_pim::types::VendorId for VendorId {
    const VENDOR_ID_UNKNOWN: Self = Self([0; 2]);
}
