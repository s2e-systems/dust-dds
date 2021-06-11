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

pub enum SubmessageElements<'a,
    PSM: EntityIdPIM + EntityIdSubmessageElementPIM<PSM> + SerializedDataSubmessageElementPIM<'a>,
> {
    EntityId(PSM::EntityIdSubmessageElementType),
    SerializedData(PSM::SerializedDataSubmessageElementType),
}

pub trait UShortSubmessageElementPIM {
    type UShortSubmessageElementType: UShortSubmessageElementType;
}

pub trait UShortSubmessageElementType {
    fn new(value: u16) -> Self;
    fn value(&self) -> &u16;
}

pub trait ShortSubmessageElementPIM {
    type ShortSubmessageElementType: ShortSubmessageElementType;
}

pub trait ShortSubmessageElementType {
    fn new(value: i16) -> Self;
    fn value(&self) -> &i16;
}

pub trait ULongSubmessageElementPIM {
    type ULongSubmessageElementType: ULongSubmessageElementType;
}

pub trait ULongSubmessageElementType {
    fn new(value: u32) -> Self;
    fn value(&self) -> &u32;
}

pub trait LongSubmessageElementPIM {
    type LongSubmessageElementType: LongSubmessageElementType;
}

pub trait LongSubmessageElementType {
    fn new(value: i32) -> Self;
    fn value(&self) -> &i32;
}

pub trait GuidPrefixSubmessageElementPIM<PSM: GuidPrefixPIM> {
    type GuidPrefixSubmessageElementType: GuidPrefixSubmessageElementType<PSM>;
}

pub trait GuidPrefixSubmessageElementType<PSM: GuidPrefixPIM> {
    fn new(value: &PSM::GuidPrefixType) -> Self;
    fn value(&self) -> &PSM::GuidPrefixType;
}

pub trait EntityIdSubmessageElementPIM<PSM: EntityIdPIM> {
    type EntityIdSubmessageElementType: EntityIdSubmessageElementType<PSM>;
}

pub trait EntityIdSubmessageElementType<PSM: EntityIdPIM> {
    fn new(value: &PSM::EntityIdType) -> Self;
    fn value(&self) -> &PSM::EntityIdType;
}

pub trait VendorIdSubmessageElementPIM<PSM: VendorIdPIM> {
    type VendorIdSubmessageElementType: VendorIdSubmessageElementType<PSM>;
}

pub trait VendorIdSubmessageElementType<PSM: VendorIdPIM> {
    fn new(value: &PSM::VendorIdType) -> Self;
    fn value(&self) -> &PSM::VendorIdType;
}

pub trait ProtocolVersionSubmessageElementPIM<PSM: ProtocolVersionPIM> {
    type ProtocolVersionSubmessageElementType: ProtocolVersionSubmessageElementType<PSM>;
}

pub trait ProtocolVersionSubmessageElementType<PSM: ProtocolVersionPIM> {
    fn new(value: &PSM::ProtocolVersionType) -> Self;
    fn value(&self) -> &PSM::ProtocolVersionType;
}

pub trait SequenceNumberSubmessageElementPIM<PSM: SequenceNumberPIM> {
    type SequenceNumberSubmessageElementType: SequenceNumberSubmessageElementType<PSM>;
}

pub trait SequenceNumberSubmessageElementType<PSM: SequenceNumberPIM> {
    fn new(value: &PSM::SequenceNumberType) -> Self;
    fn value(&self) -> &PSM::SequenceNumberType;
}

pub trait SequenceNumberSetSubmessageElementPIM<PSM: SequenceNumberPIM> {
    type SequenceNumberSetSubmessageElementType: SequenceNumberSetSubmessageElementType<PSM>;
}

pub trait SequenceNumberSetSubmessageElementType<PSM: SequenceNumberPIM> {
    fn new(base: &PSM::SequenceNumberType, set: &[PSM::SequenceNumberType]) -> Self;
    fn base(&self) -> &PSM::SequenceNumberType;
    fn set(&self) -> &[PSM::SequenceNumberType];
}

pub trait FragmentNumberSubmessageElementPIM<PSM: FragmentNumberPIM> {
    type FragmentNumberSubmessageElementType: FragmentNumberSubmessageElementType<PSM>;
}

pub trait FragmentNumberSubmessageElementType<PSM: FragmentNumberPIM> {
    fn new(value: &PSM::FragmentNumberType) -> Self;
    fn value(&self) -> &PSM::FragmentNumberType;
}

pub trait FragmentNumberSetSubmessageElementPIM<PSM: FragmentNumberPIM> {
    type FragmentNumberSetSubmessageElementType: FragmentNumberSetSubmessageElementType<PSM>;
}

pub trait FragmentNumberSetSubmessageElementType<PSM: FragmentNumberPIM> {
    fn new(base: &PSM::FragmentNumberType, set: &[PSM::FragmentNumberType]) -> Self;
    fn base(&self) -> &PSM::FragmentNumberType;
    fn set(&self) -> &[PSM::FragmentNumberType];
}

pub trait TimestampSubmessageElementPIM<PSM: TimePIM> {
    type TimestampSubmessageElementType: TimestampSubmessageElementType<PSM>;
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

pub trait ParameterListSubmessageElementPIM<PSM: ParameterIdPIM> {
    type ParameterListSubmessageElementType: ParameterListSubmessageElementType<PSM>;
}

pub trait ParameterListSubmessageElementType<PSM: ParameterIdPIM> {
    type Parameter: ParameterType<PSM>;

    fn new(parameter: &[Self::Parameter]) -> Self;
    fn parameter(&self) -> &[Self::Parameter];
}

pub trait CountSubmessageElementPIM<PSM: CountPIM> {
    type CountSubmessageElementType: CountSubmessageElementType<PSM>;
}

pub trait CountSubmessageElementType<PSM: CountPIM> {
    fn new(value: &PSM::CountType) -> Self;
    fn value(&self) -> &PSM::CountType;
}

pub trait LocatorListSubmessageElementPIM<PSM: LocatorPIM> {
    type LocatorListSubmessageElementType: LocatorListSubmessageElementType<PSM>;
}

pub trait LocatorListSubmessageElementType<PSM: LocatorPIM> {
    fn new(value: &[PSM::LocatorType]) -> Self;
    fn value(&self) -> &[PSM::LocatorType];
}

pub trait SerializedDataSubmessageElementPIM<'a> {
    type SerializedDataSubmessageElementType: SerializedDataSubmessageElementType<'a>;
}

pub trait SerializedDataSubmessageElementType<'a> {
    fn new(value: &'a [u8]) -> Self;
    fn value(&self) -> &[u8];
}

pub trait SerializedDataFragmentSubmessageElementPIM<'a> {
    type SerializedDataFragmentSubmessageElementType: SerializedDataFragmentSubmessageElementType<
        'a,
    >;
}

pub trait SerializedDataFragmentSubmessageElementType<'a> {
    fn new(value: &'a [u8]) -> Self;
    fn value(&self) -> &[u8];
}

pub trait GroupDigestSubmessageElementPIM<PSM: GroupDigestPIM> {
    type GroupDigestSubmessageElementType: GroupDigestSubmessageElementType<PSM>;
}

pub trait GroupDigestSubmessageElementType<PSM: GroupDigestPIM> {
    fn new(value: &PSM::GroupDigestType) -> Self;
    fn value(&self) -> PSM::GroupDigestType;
}
