use rust_rtps_pim::{
    messages::{
        submessage_elements::{
            CountSubmessageElementConstructor, EntityIdSubmessageElementAttributes,
            EntityIdSubmessageElementConstructor, Parameter,
            ParameterListSubmessageElementAttributes, ParameterListSubmessageElementConstructor,
            SequenceNumberSetSubmessageElementConstructor,
            SequenceNumberSubmessageElementAttributes, SequenceNumberSubmessageElementConstructor,
            SerializedDataSubmessageElementAttributes, SerializedDataSubmessageElementConstructor,
            TimestampSubmessageElementAttributes,
        },
        types::{Count, FragmentNumber, GroupDigest, Time},
    },
    structure::types::{EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId},
};

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementWrite<'a> {
    pub parameter: Vec<Parameter<'a>>,
}
impl<'a> ParameterListSubmessageElementConstructor<'a> for ParameterListSubmessageElementWrite<'a> {
    fn new<P: IntoIterator<Item = Parameter<'a>>>(parameter: P) -> Self {
        Self {
            parameter: parameter.into_iter().collect(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementRead<'a> {
    pub parameter: Vec<Parameter<'a>>,
}
impl<'a> ParameterListSubmessageElementAttributes for ParameterListSubmessageElementRead<'a> {
    fn parameter(&self) -> &[Parameter<'_>] {
        self.parameter.as_ref()
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

impl<'a> SerializedDataSubmessageElementConstructor<'a> for SerializedDataSubmessageElementPsm<'a> {
    fn new(value: &'a [u8]) -> Self {
        Self { value }
    }
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

impl<'a> SequenceNumberSetSubmessageElementConstructor<'a>
    for SequenceNumberSetSubmessageElementPsm
{
    fn new(base: SequenceNumber, set: &'a [SequenceNumber]) -> Self {
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
