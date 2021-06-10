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

pub trait UShort {
    fn new(value: u16) -> Self;
    fn value(&self) -> &u16;
}

pub trait Short {
    fn new(value: i16) -> Self;
    fn value(&self) -> &i16;
}

pub trait ULong {
    fn new(value: u32) -> Self;
    fn value(&self) -> &u32;
}

pub trait Long {
    fn new(value: i32) -> Self;
    fn value(&self) -> &i32;
}

pub trait GuidPrefix<PSM: GuidPrefixPIM> {
    fn new(value: &PSM::GuidPrefixType) -> Self;
    fn value(&self) -> &PSM::GuidPrefixType;
}

pub trait EntityId<PSM: EntityIdPIM> {
    fn new(value: &PSM::EntityIdType) -> Self;
    fn value(&self) -> &PSM::EntityIdType;
}

pub trait VendorId<PSM: VendorIdPIM> {
    fn new(value: &PSM::VendorIdType) -> Self;
    fn value(&self) -> &PSM::VendorIdType;
}

pub trait ProtocolVersion<PSM: ProtocolVersionPIM> {
    fn new(value: &PSM::ProtocolVersionType) -> Self;
    fn value(&self) -> &PSM::ProtocolVersionType;
}

pub trait SequenceNumber<PSM: SequenceNumberPIM> {
    fn new(value: &PSM::SequenceNumberType) -> Self;
    fn value(&self) -> &PSM::SequenceNumberType;
}

pub trait SequenceNumberSet<PSM: SequenceNumberPIM> {
    fn new(base: &PSM::SequenceNumberType, set: &[PSM::SequenceNumberType]) -> Self;
    fn base(&self) -> &PSM::SequenceNumberType;
    fn set(&self) -> &[PSM::SequenceNumberType];
}

pub trait FragmentNumber<PSM: FragmentNumberPIM> {
    fn new(value: &PSM::FragmentNumberType) -> Self;
    fn value(&self) -> &PSM::FragmentNumberType;
}

pub trait FragmentNumberSet<PSM: FragmentNumberPIM> {
    fn new(base: &PSM::FragmentNumberType, set: &[PSM::FragmentNumberType]) -> Self;
    fn base(&self) -> &PSM::FragmentNumberType;
    fn set(&self) -> &[PSM::FragmentNumberType];
}

pub trait Timestamp<PSM: TimePIM> {
    fn new(value: &PSM::TimeType) -> Self;
    fn value(&self) -> &PSM::TimeType;
}

pub trait Parameter<PSM: ParameterIdPIM> {
    fn parameter_id(&self) -> PSM::ParameterIdType;
    fn length(&self) -> i16;
    fn value(&self) -> &[u8];
}

pub trait ParameterList<PSM: ParameterIdPIM> {
    type Parameter: Parameter<PSM>;

    fn new(parameter: &[Self::Parameter]) -> Self;
    fn parameter(&self) -> &[Self::Parameter];
}

pub trait Count<PSM: CountPIM> {
    fn new(value: &PSM::CountType) -> Self;
    fn value(&self) -> &PSM::CountType;
}

pub trait LocatorList<PSM: LocatorPIM> {
    fn new(value: &[PSM::LocatorType]) -> Self;
    fn value(&self) -> &[PSM::LocatorType];
}

pub trait SerializedData<'a> {
    fn new(value: &'a [u8]) -> Self;
    fn value(&self) -> &[u8];
}

pub trait SerializedDataFragment<'a> {
    fn new(value: &'a [u8]) -> Self;
    fn value(&self) -> &[u8];
}

pub trait GroupDigest<PSM: GroupDigestPIM> {
    fn new(value: &PSM::GroupDigestType) -> Self;
    fn value(&self) -> PSM::GroupDigestType;
}
