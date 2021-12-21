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
    fn new(value: EntityId) -> Self;
}

pub trait EntityIdSubmessageElementAttributes {
    fn value(&self) -> &EntityId;
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
pub struct Parameter<V> {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: V,
}

impl<V> Parameter<V>
where
    V: AsRef<[u8]>,
{
    pub fn new(parameter_id: ParameterId, value: V) -> Self {
        let length = ((value.as_ref().len() + 3) & !0b11) as i16; //ceil to multiple of 4;
        Self {
            parameter_id,
            length,
            value,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElement<T> {
    pub parameter: T,
}

pub trait ParameterListSubmessageElementAttributes {
    fn parameter(&self) -> &[Parameter<&[u8]>];
}

#[derive(Debug, PartialEq, Clone)]
pub struct CountSubmessageElement {
    pub value: Count,
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
    fn value(&self) -> &[u8];
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataFragmentSubmessageElement<D> {
    pub value: D,
}

#[derive(Debug, PartialEq)]
pub struct GroupDigestSubmessageElement {
    pub value: GroupDigest,
}
