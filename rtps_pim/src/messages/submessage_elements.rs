///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::structure::types::{
    EntityId, GuidPrefix, Locator, ProtocolVersion, SequenceNumber, VendorId,
};

use super::types::{Count, FragmentNumber, GroupDigest, ParameterId, Time};

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

pub struct Parameter<'a> {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: &'a [u8],
}

pub trait ParameterListSubmessageElementType<'a> {
    type IntoIter: IntoIterator<Item = Parameter<'a>>;

    fn new(parameter: &'a [Parameter]) -> Self;
    fn parameter(&'a self) -> Self::IntoIter;
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
