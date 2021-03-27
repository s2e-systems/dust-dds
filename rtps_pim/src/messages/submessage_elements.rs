///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::{messages, RtpsPsm};

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

pub struct GuidPrefix<PSM: RtpsPsm> {
    pub value: PSM::GuidPrefix,
}

pub struct EntityId<PSM: RtpsPsm> {
    pub value: PSM::EntityId,
}

pub struct VendorId<PSM: RtpsPsm> {
    pub value: PSM::VendorId,
}

pub struct ProtocolVersion<PSM: RtpsPsm> {
    pub value: PSM::ProtocolVersion,
}

pub struct SequenceNumber<PSM: RtpsPsm> {
    pub value: PSM::SequenceNumber,
}

pub struct SequenceNumberSet<PSM: RtpsPsm> {
    pub base: PSM::SequenceNumber,
    pub set: PSM::SequenceNumberSet,
}

pub struct FragmentNumber<PSM: RtpsPsm> {
    pub value: PSM::FragmentNumber,
}

pub struct FragmentNumberSet<PSM: RtpsPsm> {
    pub base: PSM::FragmentNumber,
    pub set: PSM::FragmentNumberSet,
}

pub struct Timestamp<PSM: RtpsPsm> {
    pub value: PSM::Time,
}

pub trait Parameter {
    type ParameterId;
    fn parameter_id(&self) -> Self::ParameterId;
    fn length(&self) -> i16;
    fn value(&self) -> &[u8];
}

pub struct ParameterList<PSM: RtpsPsm> {
    pub parameter: PSM::ParameterList,
}

pub struct Count<PSM: RtpsPsm> {
    pub value: <PSM as messages::Types>::Count,
}

pub struct LocatorList<PSM: RtpsPsm> {
    pub value: PSM::LocatorList,
}

pub struct SerializedData<SerializedData: AsRef<[u8]>> {
    pub value: SerializedData,
}

pub struct SerializedDataFragment<SerializedDataFragment: AsRef<[u8]>> {
    pub value: SerializedDataFragment,
}

pub struct GroupDigest<PSM: RtpsPsm> {
    pub value: <PSM as messages::Types>::GroupDigest,
}
