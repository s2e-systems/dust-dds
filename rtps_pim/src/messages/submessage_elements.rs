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
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElement<T> {
    pub parameter: T,
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

#[derive(Debug, PartialEq)]
pub struct SerializedDataFragmentSubmessageElement<'a> {
    pub value: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct GroupDigestSubmessageElement {
    pub value: GroupDigest,
}
