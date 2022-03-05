use crate::structure::types::{EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId};

use super::types::{Count, FragmentNumber, GroupDigest, ParameterId, Time};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

#[derive(Debug, PartialEq)]
pub struct UShortSubmessageElement {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub struct ShortSubmessageElement {
    value: i16,
}

#[derive(Debug, PartialEq)]
pub struct ULongSubmessageElement {
    value: u32,
}

#[derive(Debug, PartialEq)]
pub struct LongSubmessageElementConstructor {
    value: i32,
}

#[derive(Debug, PartialEq)]
pub struct GuidPrefixSubmessageElement {
    value: GuidPrefix,
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
pub struct SequenceNumberSetSubmessageElement<S> {
    pub base: SequenceNumber,
    pub set: S,
}

#[derive(Debug, PartialEq)]
pub struct FragmentNumberSubmessageElement {
    pub value: FragmentNumber,
}

pub struct FragmentNumberSetSubmessageElement<F> {
    pub base: FragmentNumber,
    pub set: F,
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

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElement<P> {
    pub parameter: P,
}

#[derive(Debug, PartialEq)]
pub struct CountSubmessageElement {
    pub value: Count,
}

#[derive(Debug, PartialEq)]
pub struct LocatorListSubmessageElement<L> {
    pub value: L,
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
    value: GroupDigest,
}
