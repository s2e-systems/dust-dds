/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///  

use std::collections::BTreeSet;

use crate::types::{Locator};
use crate::types;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Long(pub i32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ULong(pub u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Short(pub i16);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UShort(pub u16);

// ///////// The GuidPrefix, and EntityId  ////////////////////////////////////
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct GuidPrefix(pub types::GuidPrefix);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct EntityId(pub types::EntityId);

// /////////  VendorId ////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct VendorId(pub types::VendorId);

// ///////// ProtocolVersion //////////////////////////////////////////////////
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ProtocolVersion(pub types::ProtocolVersion);

//  /////////   SequenceNumber
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct SequenceNumber(pub types::SequenceNumber);


//  /////////   SequenceNumberSet
#[derive(PartialEq, Debug)]
pub struct SequenceNumberSet {
    base: types::SequenceNumber,
    set: BTreeSet<types::SequenceNumber>,
}

impl SequenceNumberSet {
    pub fn new(base: types::SequenceNumber, set: BTreeSet<types::SequenceNumber>) -> Self {
        SequenceNumberSet {
            base,
            set,
        }
    }

    pub fn from_set(set: BTreeSet<types::SequenceNumber>) -> Self { 
        let base = *set.iter().next().unwrap_or(&0);
        Self {base, set } 
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


//  /////////   FragmentNumber
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)] 
pub struct FragmentNumber(pub crate::messages::types::FragmentNumber);

//  ////////    FragmentNumberSet

#[derive(PartialEq, Debug)]
pub struct FragmentNumberSet {
    base: FragmentNumber,
    set: BTreeSet<FragmentNumber>,
}

impl FragmentNumberSet {
    pub fn new(set: BTreeSet<FragmentNumber>) -> Self { 
        let base = *set.iter().next().unwrap_or(&FragmentNumber(0));
        Self {base, set } 
    }

    pub fn is_valid(&self) -> bool {
        let min = *self.set.iter().next().unwrap(); // First element. Must exist by the invariant
        let max = *self.set.iter().next_back().unwrap(); // Last element. Must exist by the invariant

        if min >= FragmentNumber(1) && max.0 - min.0 < 256 {
            true
        } else {
            false
        }
    }
}

// //////////// Timestamp ////////////////
#[derive(PartialEq, Debug)]
pub struct Timestamp(pub crate::messages::types::Time);

//  /////////// Count ///////////
#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub struct Count(pub crate::messages::types::Count);

impl std::ops::AddAssign<i32> for Count {
    fn add_assign(&mut self, rhs: i32) {
        *self = Count(self.0+rhs)
    }
}


// /////////// LocatorList ////////////////////////////////////////////////////
#[derive(Debug, PartialEq)]
pub struct LocatorList(pub Vec<Locator>);

//  ///////////   SerializedData   //////////////////
#[derive(PartialEq, Debug)]
pub struct SerializedData(pub Vec<u8>);

//  ///////////   SerializedDataFragment  ///////////
#[derive(PartialEq, Debug)]
pub struct SerializedDataFragment(pub Vec<u8>);

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
            base: FragmentNumber(1001),
            set:  [FragmentNumber(1001), FragmentNumber(1003)].iter().cloned().collect(),
        };
        let result = FragmentNumberSet::new([FragmentNumber(1001), FragmentNumber(1003)].iter().cloned().collect());
        assert_eq!(expected, result);
    }
}
