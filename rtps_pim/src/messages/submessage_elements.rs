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
    fn value(&self) -> &u16;
}

pub trait Short {
    fn value(&self) -> &i16;
}

pub trait ULong {
    fn value(&self) -> &u32;
}

pub trait Long {
    fn value(&self) -> &i32;
}

pub trait GuidPrefix<PSM: GuidPrefixPIM> {
    fn value(&self) -> &PSM::GuidPrefixType;
}

pub trait EntityId<PSM: EntityIdPIM> {
    fn value(&self) -> &PSM::EntityIdType;
}

pub trait VendorId<PSM: VendorIdPIM> {
    fn value(&self) -> &PSM::VendorIdType;
}

pub trait ProtocolVersion<PSM: ProtocolVersionPIM> {
    fn value(&self) -> &PSM::ProtocolVersionType;
}

pub trait SequenceNumber<PSM: SequenceNumberPIM> {
    fn value(&self) -> &PSM::SequenceNumberType;
}

pub trait SequenceNumberSet<PSM: SequenceNumberPIM> {
    type SequenceNumberVector;

    fn base(&self) -> &PSM::SequenceNumberType;
    fn set(&self) -> &Self::SequenceNumberVector;
}

pub trait FragmentNumber<PSM: FragmentNumberPIM> {
    fn value(&self) -> &PSM::FragmentNumberType;
}

pub trait FragmentNumberSet<PSM: FragmentNumberPIM> {
    type FragmentNumberVector;

    fn base(&self) -> &PSM::FragmentNumberType;
    fn set(&self) -> &Self::FragmentNumberVector;
}

pub trait Timestamp<PSM: TimePIM> {
    fn value(&self) -> &PSM::TimeType;
}

pub trait Parameter<PSM: ParameterIdPIM> {
    fn parameter_id(&self) -> PSM::ParameterIdType;
    fn length(&self) -> i16;
    fn value(&self) -> &[u8];
}

pub trait ParameterList<PSM: ParameterIdPIM> {
    type Parameter: Parameter<PSM>;
    fn parameter(&self) -> &[Self::Parameter];
}

pub trait Count<PSM: CountPIM> {
    fn value(&self) -> &PSM::CountType;
}

pub trait LocatorList<PSM: LocatorPIM> {
    fn value(&self) -> &[PSM::LocatorType];
}

pub trait SerializedData {
    fn value(&self) -> &[u8];
}

pub trait SerializedDataFragment {
    fn value(&self) -> &[u8];
}

pub trait GroupDigest<PSM: GroupDigestPIM> {
    fn value(&self) -> PSM::GroupDigestType;
}
