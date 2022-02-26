use crate::structure::types::{EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId};

use super::types::{Count, FragmentNumber, GroupDigest, ParameterId, Time};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///

pub trait UShortSubmessageElementConstructor {
    fn new(value: u16) -> Self;
}

pub trait UShortSubmessageElementAttributes {
    fn value(&self) -> u16;
}

pub trait ShortSubmessageElementConstructor {
    fn new(value: i16) -> Self;
}

pub trait ShortSubmessageElementAttributes {
    fn value(&self) -> i16;
}

pub trait ULongSubmessageElementConstructor {
    fn new(value: u32) -> Self;
}

pub trait ULongSubmessageElementAttributes {
    fn value(&self) -> u32;
}

pub trait LongSubmessageElementConstructor {
    fn new(value: i32) -> Self;
}

pub trait LongSubmessageElementAttributes {
    fn value(&self) -> i32;
}

pub trait GuidPrefixSubmessageElementConstructor {
    fn new(value: GuidPrefix) -> Self;
}

pub trait GuidPrefixSubmessageElementAttributes {
    fn value(&self) -> GuidPrefix;
}

pub trait EntityIdSubmessageElementConstructor {
    fn new(value: EntityId) -> Self;
}

pub trait EntityIdSubmessageElementAttributes {
    fn value(&self) -> EntityId;
}

pub trait VendorIdSubmessageElementConstructor {
    fn new(value: VendorId) -> Self;
}

pub trait VendorIdSubmessageElementAttributes {
    fn value(&self) -> VendorId;
}

pub trait ProtocolVersionSubmessageElementConstructor {
    fn new(value: ProtocolVersion) -> Self;
}

pub trait ProtocolVersionSubmessageElementAttributes {
    fn value(&self) -> ProtocolVersion;
}

pub trait SequenceNumberSubmessageElementConstructor {
    fn new(value: SequenceNumber) -> Self;
}

pub trait SequenceNumberSubmessageElementAttributes {
    fn value(&self) -> SequenceNumber;
}

pub trait SequenceNumberSetSubmessageElementConstructor {
    type SequenceNumberSetType: ?Sized;
    fn new(base: SequenceNumber, set: &Self::SequenceNumberSetType) -> Self;
}

pub trait SequenceNumberSetSubmessageElementAttributes {
    type SequenceNumberSetType: ?Sized;
    fn base(&self) -> SequenceNumber;
    fn set(&self) -> &Self::SequenceNumberSetType;
}

pub trait FragmentNumberSubmessageElementConstructor {
    fn new(value: FragmentNumber) -> Self;
}

pub trait FragmentNumberSubmessageElementAttributes {
    fn new(&self) -> FragmentNumber;
}

pub trait FragmentNumberSetSubmessageElementConstructor {
    type FragmentNumberSetType: ?Sized;

    fn new(base: FragmentNumber, set: &Self::FragmentNumberSetType) -> Self;
}

pub trait FragmentNumberSetSubmessageElementAttributes {
    type FragmentNumberSetType: ?Sized;

    fn base(&self) -> FragmentNumber;
    fn set(&self) -> &Self::FragmentNumberSetType;
}

pub trait TimestampSubmessageElementConstructor {
    fn new(value: Time) -> Self;
}

pub trait TimestampSubmessageElementAttributes {
    fn value(&self) -> Time;
}

pub trait ParameterConstructor {
    type ParameterValueType: ?Sized;

    fn new(parameter_id: ParameterId, length: &i16, value: &Self::ParameterValueType) -> Self;
}

pub trait ParameterAttributes {
    type ParameterValueType: ?Sized;

    fn parameter_id(&self) -> ParameterId;
    fn length(&self) -> &i16;
    fn value(&self) -> &Self::ParameterValueType;
}

pub trait ParameterListSubmessageElementConstructor {
    type ParameterListType: ?Sized;
    fn new(parameter: &Self::ParameterListType) -> Self;
}

pub trait ParameterListSubmessageElementAttributes {
    type ParameterListType: ?Sized;
    fn parameter(&self) -> &Self::ParameterListType;
}

pub trait CountSubmessageElementConstructor {
    fn new(value: Count) -> Self;
}

pub trait CountSubmessageElementAttributes {
    fn value(&self) -> Count;
}

pub trait LocatorListSubmessageElementConstructor {
    type LocatorListType: ?Sized;
    fn new(value: &Self::LocatorListType) -> Self;
}

pub trait LocatorListSubmessageElementAttributes {
    type LocatorListType: ?Sized;
    fn value(&self) -> &Self::LocatorListType;
}

pub trait SerializedDataSubmessageElementConstructor {
    type SerializedDataType: ?Sized;
    fn new(value: &Self::SerializedDataType) -> Self;
}

pub trait SerializedDataSubmessageElementAttributes {
    type SerializedDataType: ?Sized;
    fn value(&self) -> &Self::SerializedDataType;
}

pub trait SerializedDataFragmentSubmessageElementConstructor {
    type SerializedDataFragmentType: ?Sized;
    fn new(value: &Self::SerializedDataFragmentType) -> Self;
}

pub trait SerializedDataFragmentSubmessageElementAttributes {
    type SerializedDataFragmentType: ?Sized;
    fn value(&self) -> &Self::SerializedDataFragmentType;
}

pub trait GroupDigestSubmessageElementConstructor {
    fn new(value: GroupDigest) -> Self;
}

pub trait GroupDigestSubmessageElementAttributes {
    fn value(&self) -> GroupDigest;
}
