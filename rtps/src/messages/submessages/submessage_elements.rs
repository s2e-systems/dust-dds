///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///  
use std::collections::BTreeSet;

use crate::{
    messages,
    types,
};

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
    base: types::SequenceNumber,
    set: BTreeSet<types::SequenceNumber>,
}

impl SequenceNumberSet {
    pub fn new(base: types::SequenceNumber, set: BTreeSet<types::SequenceNumber>) -> Self {
        SequenceNumberSet { base, set }
    }

    pub fn from_set(set: BTreeSet<types::SequenceNumber>) -> Self {
        let base = *set.iter().next().unwrap_or(&0);
        Self { base, set }
    }

    pub fn base(&self) -> &types::SequenceNumber {
        &self.base
    }

    pub fn set(&self) -> &BTreeSet<types::SequenceNumber> {
        &self.set
    }

    pub fn is_valid(&self) -> bool {
        let min = *self.set.iter().next().unwrap(); // First element. Must exist by the invariant
        let max = *self.set.iter().next_back().unwrap(); // Last element. Must exist by the invariant

        if min >= 1 && max - min < 256 {
            true
        } else {
            false
        }
    }
}

pub type FragmentNumber = messages::types::FragmentNumber;

#[derive(PartialEq, Debug)]
pub struct FragmentNumberSet {
    base: FragmentNumber,
    set: BTreeSet<FragmentNumber>,
}

impl FragmentNumberSet {
    pub fn new(base: FragmentNumber, set: BTreeSet<FragmentNumber>) -> Self {
        Self { base, set }
    }

    pub fn from_set(set: BTreeSet<FragmentNumber>) -> Self {
        let base = *set.iter().next().unwrap_or(&0);
        Self { base, set }
    }

    pub fn base(&self) -> FragmentNumber {
        self.base
    }

    pub fn set(&self) -> &BTreeSet<FragmentNumber> {
        &self.set
    }

    pub fn is_valid(&self) -> bool {
        let min = *self.set.iter().next().unwrap(); // First element. Must exist by the invariant
        let max = *self.set.iter().next_back().unwrap(); // Last element. Must exist by the invariant

        if min >= 1 && max - min < 256 {
            true
        } else {
            false
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // /////////////////////// SequenceNumberSet Tests ////////////////////////

    #[test]
    fn sequence_number_set_constructor() {
        let expected = SequenceNumberSet {
            base: 1001,
            set: [1001, 1003].iter().cloned().collect(),
        };
        let result = SequenceNumberSet::from_set([1001, 1003].iter().cloned().collect());
        assert_eq!(expected, result);
    }

    #[test]
    fn sequence_number_set_constructor_empty_set() {
        let expected = SequenceNumberSet {
            base: 0,
            set: [].iter().cloned().collect(),
        };
        let result = SequenceNumberSet::from_set([].iter().cloned().collect());
        assert_eq!(expected, result);
    }

    // /////////////////////// FragmentNumberSet Tests ////////////////////////

    #[test]
    fn fragment_number_set_constructor() {
        let expected = FragmentNumberSet {
            base: 1001,
            set: [1001, 1003].iter().cloned().collect(),
        };
        let result = FragmentNumberSet::from_set([1001, 1003].iter().cloned().collect());
        assert_eq!(expected, result);
    }
}
