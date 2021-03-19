use crate::types::{Long, ULong};

pub struct Time {
    seconds: Long,
    fraction: ULong,
}

impl rust_rtps_pim::messages::types::Time for Time {
    const TIME_ZERO: Self = Self {
        seconds: Long(0),
        fraction: ULong(0),
    };

    const TIME_INVALID: Self = Self {
        seconds: Long(std::i32::MIN),
        fraction: ULong(0xffffffff),
    };

    const TIME_INFINITE: Self = Self {
        seconds: Long(std::i32::MIN),
        fraction: ULong(0xfffffffe),
    };
}

pub struct SubmessageKind(u8);

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
