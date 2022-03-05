use rust_rtps_pim::{
    messages::{
        submessage_elements::Parameter,
        types::{Count, FragmentNumber, GroupDigest, Time},
    },
    structure::types::{EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId},
};

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementWrite<'a> {
    pub parameter: Vec<Parameter<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementRead<'a> {
    pub parameter: Vec<Parameter<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct GuidPrefixSubmessageElementPsm {
    pub value: GuidPrefix,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EntityIdSubmessageElementPsm {
    pub value: EntityId,
}

#[derive(Debug, PartialEq)]
pub struct VendorIdSubmessageElementPsm {
    pub value: VendorId,
}

#[derive(Debug, PartialEq)]
pub struct ProtocolVersionSubmessageElementPsm {
    pub value: ProtocolVersion,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SequenceNumberSubmessageElementPsm {
    pub value: SequenceNumber,
}

#[derive(Debug, PartialEq)]
pub struct FragmentNumberSubmessageElementPsm {
    pub value: FragmentNumber,
}

#[derive(Debug, PartialEq)]
pub struct FragmentNumberSetSubmessageElementPsm<T> {
    pub base: FragmentNumber,
    pub set: T,
}

#[derive(Debug, PartialEq)]
pub struct TimestampSubmessageElementPsm {
    pub value: Time,
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataSubmessageElementPsm<'a> {
    pub value: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct SequenceNumberSetSubmessageElementPsm {
    pub base: SequenceNumber,
    pub set: Vec<SequenceNumber>,
}

#[derive(Debug, PartialEq)]
pub struct LocatorListSubmessageElementPsm<T> {
    pub value: T,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CountSubmessageElementPsm {
    pub value: Count,
}

#[derive(Debug, PartialEq)]
pub struct GroupDigestSubmessageElementPsm {
    pub value: GroupDigest,
}
