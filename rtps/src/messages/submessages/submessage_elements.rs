///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::{messages, types};

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
    pub bitmap_base: SequenceNumber,
    pub bitmap: [i32; 8],
}

pub type FragmentNumber = messages::types::FragmentNumber;

#[derive(PartialEq, Debug)]
pub struct FragmentNumberSet {
    pub bitmap_base: FragmentNumber,
    pub bitmap: [i32; 8],
}
pub type Timestamp = messages::types::Time;

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    parameter_id: messages::types::ParameterId,
    length: i16, // length is rounded up to multple of 4
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
pub type LocatorList = Vec<types::Locator>;
pub type SerializedData = Vec<u8>;
pub type SerializedDataFragment = Vec<u8>;

// pub type GroupDigest = TBD