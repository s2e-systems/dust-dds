use rust_rtps_pim::{
    messages::{
        submessage_elements::{
            CountSubmessageElementConstructor, EntityIdSubmessageElementAttributes,
            EntityIdSubmessageElementConstructor, ParameterListSubmessageElementAttributes,
            ParameterListSubmessageElementConstructor,
            SequenceNumberSetSubmessageElementConstructor,
            SequenceNumberSubmessageElementAttributes, SequenceNumberSubmessageElementConstructor,
            SerializedDataSubmessageElementAttributes, TimestampSubmessageElementAttributes,
        },
        types::{Count, FragmentNumber, GroupDigest, ParameterId, Time},
    },
    structure::types::{EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<'a> {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct ParameterOwned {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: Vec<u8>,
}
impl ParameterOwned {
    pub fn new(parameter_id: ParameterId, value: &[u8]) -> Self {
        let length = ((value.len() + 3) & !0b11) as i16; //ceil to multiple of 4;
        Self {
            parameter_id,
            length,
            value: value.to_vec(),
        }
    }
}

impl<'a> Parameter<'a> {
    pub fn new(parameter_id: ParameterId, value: &'a [u8]) -> Self {
        let length = ((value.len() + 3) & !0b11) as i16; //ceil to multiple of 4;
        Self {
            parameter_id,
            length,
            value,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementWrite<'a> {
    pub parameter: &'a [ParameterOwned],
}
impl<'a> ParameterListSubmessageElementConstructor<'a> for ParameterListSubmessageElementWrite<'a> {
    type ParameterType = ParameterOwned;

    fn new(parameter: &'a [Self::ParameterType]) -> Self {
        Self { parameter }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementRead<'a> {
    pub parameter: Vec<Parameter<'a>>,
}
impl<'a> ParameterListSubmessageElementAttributes for ParameterListSubmessageElementRead<'a> {
    type ParameterType = Parameter<'a>;

    fn parameter(&self) -> &[Self::ParameterType] {
        &self.parameter
    }
}

#[derive(Debug, PartialEq)]
pub struct GuidPrefixSubmessageElementPsm {
    pub value: GuidPrefix,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EntityIdSubmessageElementPsm {
    pub value: EntityId,
}

impl EntityIdSubmessageElementConstructor for EntityIdSubmessageElementPsm {
    fn new(value: EntityId) -> Self {
        Self { value }
    }
}

impl EntityIdSubmessageElementAttributes for EntityIdSubmessageElementPsm {
    fn value(&self) -> EntityId {
        self.value
    }
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

impl SequenceNumberSubmessageElementConstructor for SequenceNumberSubmessageElementPsm {
    fn new(value: SequenceNumber) -> Self {
        Self { value }
    }
}

impl SequenceNumberSubmessageElementAttributes for SequenceNumberSubmessageElementPsm {
    fn value(&self) -> SequenceNumber {
        self.value
    }
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

impl TimestampSubmessageElementAttributes for TimestampSubmessageElementPsm {
    fn value(&self) -> Time {
        self.value
    }
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataSubmessageElementPsm<'a> {
    pub value: &'a [u8],
}

impl<'a> SerializedDataSubmessageElementAttributes for SerializedDataSubmessageElementPsm<'a> {
    fn value(&self) -> &[u8] {
        self.value
    }
}

#[derive(Debug, PartialEq)]
pub struct SequenceNumberSetSubmessageElementPsm {
    pub base: SequenceNumber,
    pub set: Vec<SequenceNumber>,
}

impl SequenceNumberSetSubmessageElementConstructor for SequenceNumberSetSubmessageElementPsm {
    fn new(base: SequenceNumber, set: &[SequenceNumber]) -> Self {
        Self {
            base,
            set: set.to_vec(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LocatorListSubmessageElementPsm<T> {
    pub value: T,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CountSubmessageElementPsm {
    pub value: Count,
}

impl CountSubmessageElementConstructor for CountSubmessageElementPsm {
    fn new(value: Count) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq)]
pub struct GroupDigestSubmessageElementPsm {
    pub value: GroupDigest,
}
