use crate::implementation::rtps::{
    messages::types::FragmentNumber,
    types::{Locator, SequenceNumber},
};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceNumberSet {
    pub base: SequenceNumber,
    pub set: Vec<SequenceNumber>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentNumberSet {
    pub base: FragmentNumber,
    pub set: Vec<FragmentNumber>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter<'a> {
    pub parameter_id: u16,
    pub length: i16,
    pub value: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterList<'a> {
    pub parameter: Vec<Parameter<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatorList {
    pub value: Vec<Locator>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedData<'a> {
    pub value: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedDataFragment<'a> {
    pub value: &'a [u8],
}
