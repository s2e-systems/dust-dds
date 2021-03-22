pub struct ProtocolId(pub [u8; 4]);
impl rust_rtps_pim::messages::types::ProtocolId for ProtocolId {
    const PROTOCOL_RTPS: Self = Self([b'R', b'T', b'P', b'S']);
}

pub struct SubmessageFlag(pub bool);

impl rust_rtps_pim::messages::types::SubmessageFlag for SubmessageFlag {}

impl Into<bool> for SubmessageFlag {
    fn into(self) -> bool {
        self.0
    }
}

pub struct SubmessageKind(pub u8);
impl rust_rtps_pim::messages::types::SubmessageKind for SubmessageKind {
    const DATA: Self = Self(0x15);
    const GAP: Self = Self(0x08);
    const HEARTBEAT: Self = Self(0x07);
    const ACKNACK: Self = Self(0x06);
    const PAD: Self = Self(0x01);
    const INFO_TS: Self = Self(0x09);
    const INFO_REPLY: Self = Self(0x0f);
    const INFO_DST: Self = Self(0x0e);
    const INFO_SRC: Self = Self(0x0c);
    const DATA_FRAG: Self = Self(0x16);
    const NACK_FRAG: Self = Self(0x12);
    const HEARTBEAT_FRAG: Self = Self(0x13);
}

pub struct Time {
    pub seconds: i32,
    pub fraction: u32,
}
impl rust_rtps_pim::messages::types::Time for Time {
    const TIME_ZERO: Self = Self {
        seconds: 0,
        fraction: 0,
    };

    const TIME_INVALID: Self = Self {
        seconds: std::i32::MIN,
        fraction: std::u32::MAX,
    };

    const TIME_INFINITE: Self = Self {
        seconds: std::i32::MIN,
        fraction: 0xfffffffe,
    };
}

pub struct Count(pub i32);
impl rust_rtps_pim::messages::types::Count for Count {}

pub struct ParameterId(pub i16);
impl rust_rtps_pim::messages::types::ParameterId for ParameterId {}

pub struct FragmentNumber(pub u32);
impl rust_rtps_pim::messages::types::FragmentNumber for FragmentNumber {}

pub struct GroupDigest(pub [u8; 4]);
impl rust_rtps_pim::messages::types::GroupDigest for GroupDigest {}
