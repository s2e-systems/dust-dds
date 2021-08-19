///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::structure::types::{
    EntityId, GuidPrefix, Locator, ProtocolVersion, SequenceNumber, VendorId,
};

use super::types::{Count, FragmentNumber, GroupDigest, ParameterId, Time};
use core::marker::PhantomData;

pub struct UShortSubmessageElement {
    pub value: u16,
}

pub struct ShortSubmessageElement {
    pub value: u16,
}

pub struct ULongSubmessageElement {
    pub value: u32,
}

pub struct LongSubmessageElement {
    pub value: i32,
}
#[derive(Debug, PartialEq)]
pub struct GuidPrefixSubmessageElement {
    pub value: GuidPrefix,
}

#[derive(Debug, PartialEq)]
pub struct EntityIdSubmessageElement {
    pub value: EntityId,
}
#[derive(Debug, PartialEq)]
pub struct VendorIdSubmessageElement {
    pub value: VendorId,
}

#[derive(Debug, PartialEq)]
pub struct ProtocolVersionSubmessageElement {
    pub value: ProtocolVersion,
}

#[derive(Debug, PartialEq)]
pub struct SequenceNumberSubmessageElement {
    pub value: SequenceNumber,
}

#[derive(Debug, PartialEq)]
pub struct SequenceNumberSetSubmessageElement<T> {
    pub base: SequenceNumber,
    pub set: T,
}

#[derive(Debug, PartialEq)]
pub struct FragmentNumberSubmessageElement {
    pub value: FragmentNumber,
}

#[derive(Debug, PartialEq)]
pub struct FragmentNumberSetSubmessageElement<T> {
    pub base: FragmentNumber,
    pub set: T,
}

#[derive(Debug, PartialEq)]
pub struct TimestampSubmessageElement {
    pub value: Time,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<'a> {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: &'a [u8],
}
impl<'a> Parameter<'a> {
    pub fn new(parameter_id: ParameterId, value: &'a [u8]) -> Self {
        let length = ((value.len() + 3) & !0b11) as i16; //ceil to multiple of 4;
        Self {
            parameter_id,
            length,
            value,
        }
    }
    // pub fn number_of_bytes(&self) -> usize {
    //     4 + self.length as usize
    // }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElement<'a, T> {
    pub parameter: T,
    pub phantom: PhantomData<&'a ()>,
}

#[derive(Debug, PartialEq)]
pub struct CountSubmessageElement {
    pub value: Count,
}

#[derive(Debug, PartialEq)]
pub struct LocatorListSubmessageElement<T> {
    pub value: T,
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataSubmessageElement<'a> {
    pub value: &'a [u8],
}

pub struct SerializedDataFragmentSubmessageElement<'a> {
    pub value: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct GroupDigestSubmessageElement {
    pub value: GroupDigest,
}


pub trait UShortSubmessageElementType {
    fn new(value: &u16) -> Self;
    fn value(&self) -> u16;
}

pub trait ShortSubmessageElementType {
    fn new(value: &i16) -> Self;
    fn value(&self) -> i16;
}

pub trait ULongSubmessageElementType {
    fn new(value: &u32) -> Self;
    fn value(&self) -> u32;
}

pub trait LongSubmessageElementType {
    fn new(value: &i32) -> Self;
    fn value(&self) -> i32;
}

pub trait GuidPrefixSubmessageElementType {
    fn new(value: &GuidPrefix) -> Self;
    fn value(&self) -> GuidPrefix;
}

pub trait EntityIdSubmessageElementType {
    fn new(value: &EntityId) -> Self;
    fn value(&self) -> EntityId;
}

pub trait VendorIdSubmessageElementType {
    fn new(value: &VendorId) -> Self;
    fn value(&self) -> VendorId;
}

pub trait ProtocolVersionSubmessageElementType {
    fn new(value: &ProtocolVersion) -> Self;
    fn value(&self) -> ProtocolVersion;
}

pub trait SequenceNumberSubmessageElementType {
    fn new(value: &SequenceNumber) -> Self;
    fn value(&self) -> SequenceNumber;
}

pub trait SequenceNumberSetSubmessageElementType {
    type IntoIter: Iterator<Item = SequenceNumber>;

    fn new(base: &SequenceNumber, set: &[SequenceNumber]) -> Self;
    fn base(&self) -> SequenceNumber;
    fn set(&self) -> Self::IntoIter;
}

pub trait FragmentNumberSubmessageElementType {
    fn new(value: &FragmentNumber) -> Self;
    fn value(&self) -> FragmentNumber;
}

pub trait FragmentNumberSetSubmessageElementType {
    type IntoIter: IntoIterator<Item = FragmentNumber>;
    fn new(base: &FragmentNumber, set: &[FragmentNumber]) -> Self;
    fn base(&self) -> FragmentNumber;
    fn set(&self) -> Self::IntoIter;
}

pub trait TimestampSubmessageElementType {
    fn new(value: &Time) -> Self;
    fn value(&self) -> Time;
}

pub trait ParameterListSubmessageElementType<'a> {
    fn new(parameter: &'a [Parameter<'a>]) -> Self
    where
        Self: Sized;

    fn parameter(&self) -> &[Parameter<'a>];
}

pub trait CountSubmessageElementType {
    fn new(value: &Count) -> Self;
    fn value(&self) -> Count;
}

pub trait LocatorListSubmessageElementType {
    type IntoIter: IntoIterator<Item = Locator>;
    fn new(value: &[Locator]) -> Self;
    fn value(&self) -> Self::IntoIter;
}

pub trait SerializedDataSubmessageElementType<'a> {
    fn new(value: &'a [u8]) -> Self;
    fn value(&self) -> &'a [u8];
}

pub trait SerializedDataFragmentSubmessageElementType<'a> {
    fn new(value: &'a [u8]) -> Self;
    fn value(&self) -> &'a [u8];
}

pub trait GroupDigestSubmessageElementType {
    fn new(value: &GroupDigest) -> Self;
    fn value(&self) -> GroupDigest;
}
