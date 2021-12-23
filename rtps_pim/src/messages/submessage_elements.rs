///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use super::types::{Count, FragmentNumber, GroupDigest, ParameterId, Time};

pub trait UShortSubmessageElementConstructor {
    fn new(value: &u16) -> Self;
}

pub trait UShortSubmessageElementAttributes {
    fn value(&self) -> &u16;
}

pub trait ShortSubmessageElementConstructor {
    fn new(value: &i16) -> Self;
}

pub trait ShortSubmessageElementAttributes {
    fn value(&self) -> &i16;
}

pub trait ULongSubmessageElementConstructor {
    fn new(value: &u32) -> Self;
}

pub trait ULongSubmessageElementAttributes {
    fn value(&self) -> &u32;
}

pub trait LongSubmessageElementConstructor {
    fn new(value: &i32) -> Self;
}

pub trait LongSubmessageElementAttributes {
    fn value(&self) -> &i32;
}

pub trait GuidPrefixSubmessageElementConstructor {
    type GuidPrefixType: ?Sized;
    fn new(value: &Self::GuidPrefixType) -> Self;
}

pub trait GuidPrefixSubmessageElementAttributes {
    type GuidPrefixType: ?Sized;
    fn value(&self) -> &Self::GuidPrefixType;
}

pub trait EntityIdSubmessageElementConstructor {
    type EntityIdType: ?Sized;
    fn new(value: &Self::EntityIdType) -> Self;
}

pub trait EntityIdSubmessageElementAttributes {
    type EntityIdType: ?Sized;
    fn value(&self) -> &Self::EntityIdType;
}

pub trait VendorIdSubmessageElementConstructor {
    type VendorIdType: ?Sized;
    fn new(value: &Self::VendorIdType) -> Self;
}

pub trait VendorIdSubmessageElementAttributes {
    type VendorIdType: ?Sized;
    fn value(&self) -> &Self::VendorIdType;
}

pub trait ProtocolVersionSubmessageElementConstructor {
    type ProtocolVersionType: ?Sized;
    fn new(value: &Self::ProtocolVersionType) -> Self;
}

pub trait ProtocolVersionSubmessageElementAttributes {
    type ProtocolVersionType: ?Sized;
    fn value(&self) -> &Self::ProtocolVersionType;
}

pub trait SequenceNumberSubmessageElementConstructor {
    type SequenceNumberType: ?Sized;
    fn new(value: &Self::SequenceNumberType) -> Self;
}

pub trait SequenceNumberSubmessageElementAttributes {
    type SequenceNumberType: ?Sized;
    fn value(&self) -> &Self::SequenceNumberType;
}

pub trait SequenceNumberSetSubmessageElementConstructor {
    type SequenceNumberType: ?Sized;
    type SequenceNumberSetType: ?Sized;
    fn new(base: &Self::SequenceNumberType, set: &Self::SequenceNumberSetType) -> Self;
}

pub trait SequenceNumberSetSubmessageElementAttributes {
    type SequenceNumberType: ?Sized;
    type SequenceNumberSetType: ?Sized;
    fn base(&self) -> &Self::SequenceNumberType;
    fn set(&self) -> &Self::SequenceNumberSetType;
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

pub trait ParameterConstructor {
    type ParameterValueType: ?Sized;

    fn new(parameter_id: ParameterId, length: i16, value: &Self::ParameterValueType) -> Self;
}

pub trait ParameterAttributes {
    type ParameterValueType: ?Sized;

    fn parameter_id(&self) -> &ParameterId;
    fn length(&self) -> &i16;
    fn value(&self) -> &Self::ParameterValueType;
}

pub trait ParameterListSubmessageElementAttributes {
    type ParameterListType: ?Sized;
    fn parameter(&self) -> &Self::ParameterListType;
}

#[derive(Debug, PartialEq, Clone)]
pub struct CountSubmessageElement {
    pub value: Count,
}

pub trait CountSubmessageElementConstructor {
    type CountType;

    fn new(value: &Self::CountType) -> Self;
}

#[derive(Debug, PartialEq)]
pub struct LocatorListSubmessageElement<T> {
    pub value: T,
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataSubmessageElement<D> {
    pub value: D,
}

pub trait SerializedDataSubmessageElementAttributes {
    type SerializedDataType: ?Sized;
    fn value(&self) -> &Self::SerializedDataType;
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataFragmentSubmessageElement<D> {
    pub value: D,
}

#[derive(Debug, PartialEq)]
pub struct GroupDigestSubmessageElement {
    pub value: GroupDigest,
}
