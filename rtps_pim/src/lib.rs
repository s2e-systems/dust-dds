#![no_std]

pub mod behavior;
pub mod messages;
pub mod structure;
// pub mod discovery;

use core::iter::FromIterator;

pub trait RtpsPim: structure::Types + messages::Types + behavior::Types {
    // Data type which is used in the RTPS CacheChange
    type Data;

    // Special custom additions to represent lists and the parameter type
    type SequenceNumberVector: IntoIterator<Item = Self::SequenceNumber>
        + FromIterator<Self::SequenceNumber>
        + Clone;
    type LocatorVector: IntoIterator<Item = Self::Locator>;
    type FragmentNumberVector: IntoIterator<Item = Self::FragmentNumber>;
    type Parameter: messages::submessage_elements::Parameter<PSM = Self>;
    type ParameterVector: IntoIterator<Item = Self::Parameter>;
}

#[cfg(test)]
pub mod test;
#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate alloc;