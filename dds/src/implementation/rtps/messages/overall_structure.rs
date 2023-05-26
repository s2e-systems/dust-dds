use crate::implementation::rtps::types::{GuidPrefix, ProtocolVersion, VendorId};

use super::types::{ProtocolId, SubmessageFlag, SubmessageKind};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct RtpsMessageHeader {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubmessageHeaderWrite {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubmessageHeaderRead<'a> {
    data: &'a [u8],
}

impl<'a> SubmessageHeaderRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn flags(&self) -> [SubmessageFlag; 8] {
        let flags_byte = self.data[1];
        [
            self.endianness_flag(),
            flags_byte & 0b_0000_0010 != 0,
            flags_byte & 0b_0000_0100 != 0,
            flags_byte & 0b_0000_1000 != 0,
            flags_byte & 0b_0001_0000 != 0,
            flags_byte & 0b_0010_0000 != 0,
            flags_byte & 0b_0100_0000 != 0,
            flags_byte & 0b_1000_0000 != 0
        ]
    }
    pub fn submessage_length(&self) -> u16 {
        let length_bytes = [self.data[2], self.data[3]];
        match self.endianness_flag() {
            true => u16::from_le_bytes(length_bytes),
            false => u16::from_be_bytes(length_bytes),
        }
    }

    pub fn endianness_flag(&self) -> bool {
        self.data[1] & 0b_0000_0001 != 0
    }
}
