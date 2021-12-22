///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::structure::types::{EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId};

use super::types::{Count, FragmentNumber, GroupDigest, ParameterId, Time};

#[derive(Debug, PartialEq)]
pub struct UShortSubmessageElement {
    pub value: u16,
}

#[derive(Debug, PartialEq)]
pub struct ShortSubmessageElement {
    pub value: u16,
}

#[derive(Debug, PartialEq)]
pub struct ULongSubmessageElement {
    pub value: u32,
}

#[derive(Debug, PartialEq)]
pub struct LongSubmessageElement {
    pub value: i32,
}

#[derive(Debug, PartialEq)]
pub struct GuidPrefixSubmessageElement {
    pub value: GuidPrefix,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EntityIdSubmessageElement {
    pub value: EntityId,
}

pub trait EntityIdSubmessageElementConstructor {
    type EntityIdType: ?Sized;
    fn new(value: &Self::EntityIdType) -> Self;
}

pub trait EntityIdSubmessageElementAttributes {
    type EntityIdType: ?Sized;
    fn value(&self) -> &Self::EntityIdType;
}

#[derive(Debug, PartialEq)]
pub struct VendorIdSubmessageElement {
    pub value: VendorId,
}

#[derive(Debug, PartialEq)]
pub struct ProtocolVersionSubmessageElement {
    pub value: ProtocolVersion,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SequenceNumberSubmessageElement {
    pub value: SequenceNumber,
}

pub trait SequenceNumberSubmessageElementAttributes {
    fn value(&self) -> &SequenceNumber;
}

pub trait SequenceNumberSetSubmessageElementConstructor {
    fn new(base: SequenceNumber, set: &[SequenceNumber]) -> Self;
}

pub trait SequenceNumberSetSubmessageElementAttributes {
    fn base(&self) -> &SequenceNumber;
    fn set(&self) -> &[SequenceNumber];
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
