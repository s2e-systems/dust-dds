///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::{messages, structure};

pub struct UShort {
    pub value: u16,
}

pub struct Short {
    pub value: i16,
}

pub struct ULong {
    pub value: u32,
}

pub struct Long {
    pub value: i32,
}

pub struct GuidPrefix<PSM: structure::Types> {
    pub value: PSM::GuidPrefix,
}

pub struct EntityId<PSM: structure::Types> {
    pub value: PSM::EntityId,
}

pub struct VendorId<PSM: structure::Types> {
    pub value: PSM::VendorId,
}

pub struct ProtocolVersion<PSM: structure::Types> {
    pub value: PSM::ProtocolVersion,
}

pub struct SequenceNumber<PSM: structure::Types> {
    pub value: PSM::SequenceNumber,
}

pub struct SequenceNumberSet<PSM: structure::Types> {
    pub base: PSM::SequenceNumber,
    pub set: PSM::SequenceNumberVector,
}

pub struct FragmentNumber<PSM: messages::Types> {
    pub value: PSM::FragmentNumber,
}

pub struct FragmentNumberSet<PSM: messages::Types> {
    pub base: PSM::FragmentNumber,
    pub set: PSM::FragmentNumberVector,
}

pub struct Timestamp<PSM: messages::Types> {
    pub value: PSM::Time,
}

pub trait Parameter {
    type PSM: messages::Types;
    fn parameter_id(&self) -> <Self::PSM as messages::Types>::ParameterId;
    fn length(&self) -> i16;
    fn value(&self) -> &[u8];
}

pub struct ParameterList<PSM: structure::Types> {
    pub parameter: PSM::ParameterVector,
}

pub struct Count<PSM: messages::Types> {
    pub value: PSM::Count,
}

pub struct LocatorList<PSM: structure::Types> {
    pub value: PSM::LocatorVector,
}

pub struct SerializedData<SerializedData: AsRef<[u8]>> {
    pub value: SerializedData,
}

pub struct SerializedDataFragment<SerializedDataFragment: AsRef<[u8]>> {
    pub value: SerializedDataFragment,
}

pub struct GroupDigest<PSM: messages::Types> {
    pub value: PSM::GroupDigest,
}
