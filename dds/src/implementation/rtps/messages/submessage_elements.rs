use crate::implementation::rtps::types::{Locator, EntityId, GuidPrefix, VendorId, ProtocolVersion, Count, SequenceNumber};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UShortSubmessageElement {
    pub value: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ULongSubmessageElement {
    pub value: u32,
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
    pub value: VendorId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolVersionSubmessageElement {
    pub value: ProtocolVersion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceNumberSetSubmessageElement {
    pub base: SequenceNumber,
    pub set: Vec<SequenceNumber>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentNumberSet {
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
