#![no_std]

pub mod behavior;
pub mod messages;
pub mod structure;
// pub mod discovery;

use core::iter::FromIterator;

pub trait RtpsPsm: structure::Types + messages::Types + behavior::Types {
    // Special custom additions to represent lists and the parameter type
    type SequenceNumberSet: IntoIterator<Item = Self::SequenceNumber>
        + FromIterator<Self::SequenceNumber>
        + Clone;
    type LocatorList: IntoIterator<Item = Self::Locator>;
    type FragmentNumberSet: IntoIterator<Item = Self::FragmentNumber>;
    type Parameter: messages::submessage_elements::Parameter<PSM = Self>;
    type ParameterList: IntoIterator<Item = Self::Parameter>;
}

#[cfg(test)]
pub mod test;
#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate alloc;