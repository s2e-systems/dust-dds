use crate::implementation::rtps::types::{Locator, EntityId, GuidPrefix};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UShortSubmessageElement {
    pub value: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortSubmessageElement {
    pub value: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ULongSubmessageElement {
    pub value: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LongSubmessageElementConstructor {
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuidPrefixSubmessageElement {
    pub value: GuidPrefix,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityIdSubmessageElement {
    pub value: EntityId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VendorIdSubmessageElement {
    pub value: [u8; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolVersionSubmessageElement {
    pub value: [u8; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceNumberSubmessageElement {
    pub value: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceNumberSetSubmessageElement {
    pub base: i64,
    pub set: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentNumberSubmessageElement {
    pub value: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentNumberSetSubmessageElement {
    pub base: u32,
    pub set: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampSubmessageElement {
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter<'a> {
    pub parameter_id: u16,
    pub length: i16,
    pub value: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterListSubmessageElement<'a> {
    pub parameter: Vec<Parameter<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountSubmessageElement {
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatorListSubmessageElement {
    pub value: Vec<Locator>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedDataSubmessageElement<'a> {
    pub value: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedDataFragmentSubmessageElement<'a> {
    pub value: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupDigestSubmessageElement {
    pub value: [u8; 4],
}
