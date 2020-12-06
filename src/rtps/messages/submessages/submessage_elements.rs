/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///  

use std::collections::BTreeSet;

use crate::rtps::types;
use crate::rtps::messages;

pub type Long = i32;

pub type ULong = u32;

pub type Short = i16;

pub type UShort = u16;

// ///////// The GuidPrefix, and EntityId  ////////////////////////////////////
pub type GuidPrefix = types::GuidPrefix;

pub type EntityId = types::EntityId;

// /////////  VendorId ////////////////////////////////////////////////////////
pub type VendorId = types::VendorId;

// ///////// ProtocolVersion //////////////////////////////////////////////////
pub type ProtocolVersion = types::ProtocolVersion;

//  /////////   SequenceNumber
pub type SequenceNumber = crate::types::SequenceNumber;

pub type ParameterList = crate::types::ParameterList;

//  /////////   SequenceNumberSet
#[derive(PartialEq, Debug)]
pub struct SequenceNumberSet {
    base: crate::types::SequenceNumber,
    set: BTreeSet<crate::types::SequenceNumber>,
}

impl SequenceNumberSet {
    pub fn new(base: crate::types::SequenceNumber, set: BTreeSet<crate::types::SequenceNumber>) -> Self {
        SequenceNumberSet {
            base,
            set,
        }
    }

    pub fn from_set(set: BTreeSet<crate::types::SequenceNumber>) -> Self { 
        let base = *set.iter().next().unwrap_or(&0);
        Self {base, set } 
    }

    pub fn base(&self) -> &crate::types::SequenceNumber {
        &self.base
    }

    pub fn set(&self) -> &BTreeSet<crate::types::SequenceNumber> {
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


//  /////////   FragmentNumber
pub type FragmentNumber = messages::types::FragmentNumber;

//  ////////    FragmentNumberSet

#[derive(PartialEq, Debug)]
pub struct FragmentNumberSet {
    base: FragmentNumber,
    set: BTreeSet<FragmentNumber>,
}

impl FragmentNumberSet {
    pub fn new(base: FragmentNumber, set: BTreeSet<FragmentNumber>) -> Self {
        Self{
            base,
            set
        }
    }

    pub fn from_set(set: BTreeSet<FragmentNumber>) -> Self { 
        let base = *set.iter().next().unwrap_or(&0);
        Self {base, set } 
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

        if min >= 1 && max- min < 256 {
            true
        } else {
            false
        }
    }
}

// //////////// Timestamp ////////////////
pub type Timestamp = messages::types::Time;

//  /////////// Count ///////////
pub type Count = messages::types::Count;


// /////////// LocatorList ////////////////////////////////////////////////////
pub type LocatorList = Vec<types::Locator>;

//  ///////////   SerializedData   //////////////////
pub type SerializedData = Vec<u8>;

//  ///////////   SerializedDataFragment  ///////////
pub type SerializedDataFragment = Vec<u8>;

//  ///////////   GroupDigest   //////////////////////
// todo


#[cfg(test)]
mod tests {
    use super::*;

    // /////////////////////// SequenceNumberSet Tests ////////////////////////

    #[test]
    fn sequence_number_set_constructor() {
        let expected = SequenceNumberSet{
            base: 1001,
            set:  [1001, 1003].iter().cloned().collect(),
        };
        let result = SequenceNumberSet::from_set([1001, 1003].iter().cloned().collect());
        assert_eq!(expected, result);
    }

    #[test]
    fn sequence_number_set_constructor_empty_set() {        
        let expected = SequenceNumberSet{
            base: 0,
            set:  [].iter().cloned().collect(),
        };
        let result = SequenceNumberSet::from_set([].iter().cloned().collect());
        assert_eq!(expected, result);
    }

    // /////////////////////// FragmentNumberSet Tests ////////////////////////

    #[test]
    fn fragment_number_set_constructor() {
        let expected = FragmentNumberSet{
            base: 1001,
            set:  [1001, 1003].iter().cloned().collect(),
        };
        let result = FragmentNumberSet::from_set([1001, 1003].iter().cloned().collect());
        assert_eq!(expected, result);
    }
}
