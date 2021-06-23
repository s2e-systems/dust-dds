use structure::types::{EntityIdPIM, GuidPrefixPIM};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::structure::{
    self,
    types::{LocatorPIM, ProtocolVersionPIM, SequenceNumberPIM, VendorIdPIM},
};

use super::types::{CountPIM, FragmentNumberPIM, GroupDigestPIM, ParameterIdPIM, TimePIM};

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

pub trait GuidPrefixSubmessageElementPIM<PSM> {
    type GuidPrefixSubmessageElementType;
}

pub trait GuidPrefixSubmessageElementType<PSM: GuidPrefixPIM> {
    fn new(value: &PSM::GuidPrefixType) -> Self;
    fn value(&self) -> &PSM::GuidPrefixType;
}

pub trait EntityIdSubmessageElementPIM<PSM> {
    type EntityIdSubmessageElementType;
}

pub trait EntityIdSubmessageElementType<PSM: EntityIdPIM> {
    fn new(value: &PSM::EntityIdType) -> Self;
    fn value(&self) -> &PSM::EntityIdType;
}

pub trait VendorIdSubmessageElementPIM<PSM> {
    type VendorIdSubmessageElementType;
}

pub trait VendorIdSubmessageElementType<PSM: VendorIdPIM> {
    fn new(value: &PSM::VendorIdType) -> Self;
    fn value(&self) -> &PSM::VendorIdType;
}

pub trait ProtocolVersionSubmessageElementPIM<PSM> {
    type ProtocolVersionSubmessageElementType;
}

pub trait ProtocolVersionSubmessageElementType<PSM: ProtocolVersionPIM> {
    fn new(value: &PSM::ProtocolVersionType) -> Self;
    fn value(&self) -> &PSM::ProtocolVersionType;
}

pub trait SequenceNumberSubmessageElementPIM<PSM> {
    type SequenceNumberSubmessageElementType;
}

pub trait SequenceNumberSubmessageElementType<PSM: SequenceNumberPIM> {
    fn new(value: &PSM::SequenceNumberType) -> Self;
    fn value(&self) -> &PSM::SequenceNumberType;
}

pub trait SequenceNumberSetSubmessageElementPIM<PSM> {
    type SequenceNumberSetSubmessageElementType;
}

pub trait SequenceNumberSetSubmessageElementType<PSM: SequenceNumberPIM> {
    fn new(base: &PSM::SequenceNumberType, set: &[PSM::SequenceNumberType]) -> Self;
    fn base(&self) -> &PSM::SequenceNumberType;
    fn set(&self) -> &[PSM::SequenceNumberType];
}

pub trait FragmentNumberSubmessageElementPIM<PSM> {
    type FragmentNumberSubmessageElementType;
}

pub trait FragmentNumberSubmessageElementType<PSM: FragmentNumberPIM> {
    fn new(value: &PSM::FragmentNumberType) -> Self;
    fn value(&self) -> &PSM::FragmentNumberType;
}

pub trait FragmentNumberSetSubmessageElementPIM<PSM> {
    type FragmentNumberSetSubmessageElementType;
}

pub trait FragmentNumberSetSubmessageElementType<PSM: FragmentNumberPIM> {
    fn new(base: &PSM::FragmentNumberType, set: &[PSM::FragmentNumberType]) -> Self;
    fn base(&self) -> &PSM::FragmentNumberType;
    fn set(&self) -> &[PSM::FragmentNumberType];
}

pub trait TimestampSubmessageElementPIM<PSM> {
    type TimestampSubmessageElementType;
}

pub trait TimestampSubmessageElementType<PSM: TimePIM> {
    fn new(value: &PSM::TimeType) -> Self;
    fn value(&self) -> &PSM::TimeType;
}

pub trait ParameterType<PSM: ParameterIdPIM> {
    fn parameter_id(&self) -> PSM::ParameterIdType;
    fn length(&self) -> i16;
    fn value(&self) -> &[u8];
}

pub trait ParameterListSubmessageElementPIM<PSM> {
    type ParameterListSubmessageElementType;
}

pub trait ParameterListSubmessageElementType<PSM> {
    type Parameter;

    fn new(parameter: &[Self::Parameter]) -> Self;
    fn parameter(&self) -> &[Self::Parameter];
}

pub trait CountSubmessageElementPIM<PSM> {
    type CountSubmessageElementType;
}

pub trait CountSubmessageElementType<PSM: CountPIM> {
    fn new(value: &PSM::CountType) -> Self;
    fn value(&self) -> &PSM::CountType;
}

pub trait LocatorListSubmessageElementPIM<PSM> {
    type LocatorListSubmessageElementType;
}

pub trait LocatorListSubmessageElementType<PSM: LocatorPIM> {
    fn new(value: &[PSM::LocatorType]) -> Self;
    fn value(&self) -> &[PSM::LocatorType];
}

pub trait SerializedDataSubmessageElementPIM<'a> {
    type SerializedDataSubmessageElementType;
}

pub trait SerializedDataSubmessageElementType<'a> {
    fn new(value: &'a [u8]) -> Self;
    fn value(&self) -> &[u8];
}

pub trait SerializedDataFragmentSubmessageElementPIM<'a> {
    type SerializedDataFragmentSubmessageElementType;
}

pub trait SerializedDataFragmentSubmessageElementType<'a> {
    fn new(value: &'a [u8]) -> Self;
    fn value(&self) -> &[u8];
}

pub trait GroupDigestSubmessageElementPIM<PSM> {
    type GroupDigestSubmessageElementType;
}

pub trait GroupDigestSubmessageElementType<PSM: GroupDigestPIM> {
    fn new(value: &PSM::GroupDigestType) -> Self;
    fn value(&self) -> PSM::GroupDigestType;
}
