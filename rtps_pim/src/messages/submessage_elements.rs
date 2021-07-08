///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::structure::types::{EntityId, GuidPrefix, Locator, SequenceNumber, VendorId};

use super::types::{CountPIM, FragmentNumber, GroupDigestPIM, ParameterId, TimePIM};

pub trait UShortSubmessageElementType {
    fn new(value: u16) -> Self;
    fn value(&self) -> &u16;
}

pub trait ShortSubmessageElementType {
    fn new(value: i16) -> Self;
    fn value(&self) -> &i16;
}

pub trait ULongSubmessageElementType {
    fn new(value: u32) -> Self;
    fn value(&self) -> &u32;
}

pub trait LongSubmessageElementType {
    fn new(value: i32) -> Self;
    fn value(&self) -> &i32;
}

pub trait GuidPrefixSubmessageElementType {
    fn new(value: &GuidPrefix) -> Self;
    fn value(&self) -> &GuidPrefix;
}

pub trait EntityIdSubmessageElementType {
    fn new(value: &EntityId) -> Self;
    fn value(&self) -> &EntityId;
}

pub trait VendorIdSubmessageElementType {
    fn new(value: &VendorId) -> Self;
    fn value(&self) -> &VendorId;
}

pub trait ProtocolVersionSubmessageElementType {
    type ProtocolVersionType;
    const PROTOCOLVERSION_1_0: Self::ProtocolVersionType;
    const PROTOCOLVERSION_1_1: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_0: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_1: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_2: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_3: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_4: Self::ProtocolVersionType;

    fn new(value: &Self::ProtocolVersionType) -> Self;
    fn value(&self) -> &Self::ProtocolVersionType;
}

pub trait SequenceNumberSubmessageElementType {
    fn new(value: SequenceNumber) -> Self;
    fn value(&self) -> SequenceNumber;
}

pub trait SequenceNumberSetSubmessageElementType {
    type IntoIter: Iterator<Item = SequenceNumber>;

    fn new(base: SequenceNumber, set: &[SequenceNumber]) -> Self;
    fn base(&self) -> SequenceNumber;
    fn set(&self) -> Self::IntoIter;
}

pub trait FragmentNumberSubmessageElementType {
    fn new(value: &FragmentNumber) -> Self;
    fn value(&self) -> &FragmentNumber;
}

pub trait FragmentNumberSetSubmessageElementType {
    fn new(base: &FragmentNumber, set: &[FragmentNumber]) -> Self;
    fn base(&self) -> &FragmentNumber;
    fn set(&self) -> &[FragmentNumber];
}

pub trait TimestampSubmessageElementType<PSM: TimePIM> {
    fn new(value: &PSM::TimeType) -> Self;
    fn value(&self) -> &PSM::TimeType;
}

pub trait ParameterType {
    fn parameter_id(&self) -> ParameterId;
    fn length(&self) -> i16;
    fn value(&self) -> &[u8];
}

pub trait ParameterListSubmessageElementType {
    type Parameter;

    fn new(parameter: &[Self::Parameter]) -> Self;
    fn empty() -> Self;
    fn parameter(&self) -> &[Self::Parameter];
}

pub trait CountSubmessageElementType<PSM: CountPIM> {
    fn new(value: &PSM::CountType) -> Self;
    fn value(&self) -> &PSM::CountType;
}

pub trait LocatorListSubmessageElementType {
    fn new(value: &[Locator]) -> Self;
    fn value(&self) -> &[Locator];
}

// pub trait SerializedDataSubmessageElementPIM {
//     type SerializedDataSubmessageElementType: SerializedDataSubmessageElementType;
// }

pub trait SerializedDataSubmessageElementType<'a> {
    type Value: ?Sized;
    type Constructed;
    fn new(value: &Self::Value) -> Self::Constructed;
    fn value(&self) -> &Self::Value;
}

pub trait SerializedDataFragmentSubmessageElementType {
    type Value;
    fn new<T: Into<Self::Value>>(value: T) -> Self;
    fn value(&self) -> &[u8];
}

pub trait GroupDigestSubmessageElementType<PSM: GroupDigestPIM> {
    fn new(value: &PSM::GroupDigestType) -> Self;
    fn value(&self) -> PSM::GroupDigestType;
}
