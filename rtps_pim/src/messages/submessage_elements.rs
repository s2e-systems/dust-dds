///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::structure::types::{EntityId, GuidPrefix, Locator, SequenceNumber, VendorId};

use super::types::{CountPIM, FragmentNumber, GroupDigestPIM, ParameterId, TimePIM};

pub trait UShortSubmessageElementPIM {
    type UShortSubmessageElementType;
}

pub trait UShortSubmessageElementType {
    fn new(value: u16) -> Self;
    fn value(&self) -> &u16;
}

pub trait ShortSubmessageElementPIM {
    type ShortSubmessageElementType;
}

pub trait ShortSubmessageElementType {
    fn new(value: i16) -> Self;
    fn value(&self) -> &i16;
}

pub trait ULongSubmessageElementPIM {
    type ULongSubmessageElementType;
}

pub trait ULongSubmessageElementType {
    fn new(value: u32) -> Self;
    fn value(&self) -> &u32;
}

pub trait LongSubmessageElementPIM {
    type LongSubmessageElementType;
}

pub trait LongSubmessageElementType {
    fn new(value: i32) -> Self;
    fn value(&self) -> &i32;
}

pub trait GuidPrefixSubmessageElementPIM {
    type GuidPrefixSubmessageElementType;
}

pub trait GuidPrefixSubmessageElementType {
    fn new(value: &GuidPrefix) -> Self;
    fn value(&self) -> &GuidPrefix;
}

pub trait EntityIdSubmessageElementPIM {
    type EntityIdSubmessageElementType;
}

pub trait EntityIdSubmessageElementType {
    fn new(value: &EntityId) -> Self;
    fn value(&self) -> &EntityId;
}

pub trait VendorIdSubmessageElementPIM {
    type VendorIdSubmessageElementType;
}

pub trait VendorIdSubmessageElementType {
    fn new(value: &VendorId) -> Self;
    fn value(&self) -> &VendorId;
}

pub trait ProtocolVersionSubmessageElementPIM {
    type ProtocolVersionSubmessageElementType;
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

pub trait SequenceNumberSubmessageElementPIM {
    type SequenceNumberSubmessageElementType;
}

pub trait SequenceNumberSubmessageElementType {
    fn new(value: SequenceNumber) -> Self;
    fn value(&self) -> SequenceNumber;
}

pub trait SequenceNumberSetSubmessageElementPIM {
    type SequenceNumberSetSubmessageElementType;
}

pub trait SequenceNumberSetSubmessageElementType {
    type IntoIter: Iterator<Item = SequenceNumber>;

    fn new(base: SequenceNumber, set: &[SequenceNumber]) -> Self;
    fn base(&self) -> SequenceNumber;
    fn set(&self) -> Self::IntoIter;
}

pub trait FragmentNumberSubmessageElementPIM {
    type FragmentNumberSubmessageElementType;
}

pub trait FragmentNumberSubmessageElementType {
    fn new(value: &FragmentNumber) -> Self;
    fn value(&self) -> &FragmentNumber;
}

pub trait FragmentNumberSetSubmessageElementPIM {
    type FragmentNumberSetSubmessageElementType;
}

pub trait FragmentNumberSetSubmessageElementType {
    fn new(base: &FragmentNumber, set: &[FragmentNumber]) -> Self;
    fn base(&self) -> &FragmentNumber;
    fn set(&self) -> &[FragmentNumber];
}

pub trait TimestampSubmessageElementPIM {
    type TimestampSubmessageElementType;
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

pub trait ParameterListSubmessageElementPIM {
    type ParameterListSubmessageElementType;
}

pub trait ParameterListSubmessageElementType {
    type Parameter;

    fn new(parameter: &[Self::Parameter]) -> Self;
    fn empty() -> Self;
    fn parameter(&self) -> &[Self::Parameter];
}

pub trait CountSubmessageElementPIM {
    type CountSubmessageElementType;
}

pub trait CountSubmessageElementType<PSM: CountPIM> {
    fn new(value: &PSM::CountType) -> Self;
    fn value(&self) -> &PSM::CountType;
}

pub trait LocatorListSubmessageElementPIM {
    type LocatorListSubmessageElementType;
}

pub trait LocatorListSubmessageElementType {
    fn new(value: &[Locator]) -> Self;
    fn value(&self) -> &[Locator];
}

pub trait SerializedDataSubmessageElementPIM {
    type SerializedDataSubmessageElementType: SerializedDataSubmessageElementType;
}

pub trait SerializedDataSubmessageElementType {
    type Value;
    fn new<T: Into<Self::Value>>(value: T) -> Self;
    fn value(&self) -> &Self::Value;
}

pub trait SerializedDataFragmentSubmessageElementPIM {
    type SerializedDataFragmentSubmessageElementType;
}

pub trait SerializedDataFragmentSubmessageElementType {
    type Value;
    fn new<T: Into<Self::Value>>(value: T) -> Self;
    fn value(&self) -> &[u8];
}

pub trait GroupDigestSubmessageElementPIM {
    type GroupDigestSubmessageElementType;
}

pub trait GroupDigestSubmessageElementType<PSM: GroupDigestPIM> {
    fn new(value: &PSM::GroupDigestType) -> Self;
    fn value(&self) -> PSM::GroupDigestType;
}
