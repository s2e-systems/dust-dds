///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

use crate::{messages, types};
use serde::ser::SerializeStruct;

pub type Long = i32;
pub type ULong = u32;
pub type Short = i16;
pub type UShort = u16;

pub type GuidPrefix = types::GuidPrefix;
pub type EntityId = types::EntityId;
pub type VendorId = types::VendorId;
pub type ProtocolVersion = types::ProtocolVersion;

pub type SequenceNumber = types::SequenceNumber;

#[derive(PartialEq, Debug)]
pub struct SequenceNumberSet {
    bitmap_base: SequenceNumber,
    num_bits: u32,
    bitmap: [Long; 8],
}

impl serde::Serialize for SequenceNumberSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SequenceNumberSet", 3)?;
        state.serialize_field("bitmap_base", &self.bitmap_base)?;
        state.serialize_field("num_bits", &self.num_bits)?;
        let range = ((self.num_bits + 31) / 32) as usize;
        for i in 0..range {
            state.serialize_field("bitmap", &self.bitmap[i])?;
        }
        state.end()
    }
}

impl SequenceNumberSet {
    pub fn new(bitmap_base: SequenceNumber, num_bits: u32, bitmap: [Long; 8]) -> Self {
        Self {
            bitmap_base,
            num_bits,
            bitmap,
        }
    }

    pub fn bitmap_base(&self) -> SequenceNumber {
        self.bitmap_base
    }

    pub fn num_bits(&self) -> u32 {
        self.num_bits
    }

    pub fn bitmap(&self) -> &[Long; 8] {
        &self.bitmap
    }
}

pub type FragmentNumber = messages::types::FragmentNumber;

#[derive(PartialEq, Debug)]
pub struct FragmentNumberSet {
    pub bitmap_base: FragmentNumber,
    pub bitmap: [Long; 8],
}

impl FragmentNumberSet {
    pub fn new(bitmap_base: FragmentNumber, bitmap: [Long; 8]) -> Self {
        Self {
            bitmap_base,
            bitmap,
        }
    }
}

pub type Timestamp = messages::types::Time;

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    parameter_id: messages::types::ParameterId,
    length: i16, // length is rounded up to multiple of 4
    value: Vec<u8>,
}

impl Parameter {
    pub fn new(parameter_id: messages::types::ParameterId, value: Vec<u8>) -> Self {
        Self {
            parameter_id,
            length: (value.len() + 3 & !3) as i16,
            value,
        }
    }

    pub fn parameter_id(&self) -> messages::types::ParameterId {
        self.parameter_id
    }

    pub fn length(&self) -> i16 {
        self.length
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParameterList {
    pub parameter: Vec<Parameter>,
}

impl ParameterList {
    pub fn new() -> Self {
        Self {
            parameter: Vec::new(),
        }
    }
}

pub type Count = messages::types::Count;
pub type LocatorList = [types::Locator; 8];
pub type SerializedData = [u8];
pub type SerializedDataFragment = [u8];

// pub type GroupDigest = TBD

#[cfg(test)]
mod tests {
    use serde_test::{assert_ser_tokens, Token};

    use super::*;

    #[test]
    fn serialize_sequence_number_set() {
        assert_ser_tokens(
            &SequenceNumberSet::new(8.into(), 16, [8, 0, 0, 0, 0, 0, 0, 0]),
            &[
                Token::Struct {
                    name: "SequenceNumberSet",
                    len: 3,
                },
                Token::Str("bitmap_base"),
                Token::Struct {
                    name: "SequenceNumber",
                    len: 2,
                },
                Token::Str("high"),
                Token::I32(0),
                Token::Str("low"),
                Token::U32(8),
                Token::StructEnd,
                Token::Str("num_bits"),
                Token::U32(16),
                Token::Str("bitmap"),
                Token::I32(8),
                Token::StructEnd,
            ],
        );

        assert_ser_tokens(
            &SequenceNumberSet::new(8.into(), 128, [8, 9, 10, 11, 0, 0, 0, 0]),
            &[
                Token::Struct {
                    name: "SequenceNumberSet",
                    len: 3,
                },
                Token::Str("bitmap_base"),
                Token::Struct {
                    name: "SequenceNumber",
                    len: 2,
                },
                Token::Str("high"),
                Token::I32(0),
                Token::Str("low"),
                Token::U32(8),
                Token::StructEnd,
                Token::Str("num_bits"),
                Token::U32(128),
                Token::Str("bitmap"),
                Token::I32(8),
                Token::Str("bitmap"),
                Token::I32(9),
                Token::Str("bitmap"),
                Token::I32(10),
                Token::Str("bitmap"),
                Token::I32(11),
                Token::StructEnd,
            ],
        )
    }
}
