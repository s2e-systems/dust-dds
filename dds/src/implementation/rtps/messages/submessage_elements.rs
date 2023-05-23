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
pub struct LocatorList {
    value: Vec<Locator>,
}

impl LocatorList {
    pub fn new(value: Vec<Locator>) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &[Locator] {
        self.value.as_ref()
    }
}
