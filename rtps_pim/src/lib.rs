// #![no_std]

pub mod behavior;
pub mod messages;
pub mod structure;
// pub mod discovery;

pub trait RtpsPsm: structure::Types + messages::Types + behavior::Types {
    // Special additions to represent lists and the parameter type
    type SequenceNumberSet: IntoIterator<Item = Self::SequenceNumber>;
    type LocatorList: IntoIterator<Item = Self::Locator>;
    type FragmentNumberSet: IntoIterator<Item = Self::FragmentNumber>;
    type Parameter: messages::submessage_elements::Parameter<ParameterId = Self::ParameterId>;
    type ParameterList: IntoIterator<Item = Self::Parameter>;
}
