use alloc::vec::Vec;

use crate::types::Locator;

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

#[derive(Clone, Debug, PartialEq)]
pub struct UShortSubmessageElement {
    pub value: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShortSubmessageElement {
    pub value: i16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ULongSubmessageElement {
    pub value: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LongSubmessageElementConstructor {
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GuidPrefixSubmessageElement {
    pub value: [u8; 12],
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityIdSubmessageElement {
    pub value: [u8; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct VendorIdSubmessageElement {
    pub value: [u8; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtocolVersionSubmessageElement {
    pub value: [u8; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct SequenceNumberSubmessageElement {
    pub value: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SequenceNumberSetSubmessageElement {
    pub base: i64,
    pub set: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentNumberSubmessageElement {
    pub value: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FragmentNumberSetSubmessageElement {
    pub base: u32,
    pub set: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimestampSubmessageElement {
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter<'a> {
    pub parameter_id: u16,
    pub length: i16,
    pub value: &'a [u8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterListSubmessageElement<'a> {
    pub parameter: Vec<Parameter<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CountSubmessageElement {
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocatorListSubmessageElement {
    pub value: Vec<Locator>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SerializedDataSubmessageElement<'a> {
    pub value: &'a [u8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct SerializedDataFragmentSubmessageElement<'a> {
    pub value: &'a [u8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupDigestSubmessageElement {
    pub value: [u8; 4],
}
