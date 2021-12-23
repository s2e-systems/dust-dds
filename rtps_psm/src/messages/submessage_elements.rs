use rust_rtps_pim::{
    messages::{
        submessage_elements::{
            CountSubmessageElementConstructor, EntityIdSubmessageElementAttributes,
            EntityIdSubmessageElementConstructor, ParameterListSubmessageElementAttributes,
            SequenceNumberSetSubmessageElementConstructor,
            SequenceNumberSubmessageElementAttributes, SequenceNumberSubmessageElementConstructor,
            SerializedDataSubmessageElementAttributes, TimestampSubmessageElementAttributes,
        },
        types::{Count, FragmentNumber, GroupDigest, ParameterId, Time},
    },
    structure::types::{EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<V> {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: V,
}

impl<V> Parameter<V>
where
    V: AsRef<[u8]>,
{
    pub fn new(parameter_id: ParameterId, value: V) -> Self {
        let length = ((value.as_ref().len() + 3) & !0b11) as i16; //ceil to multiple of 4;
        Self {
            parameter_id,
            length,
            value,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElement<T> {
    pub parameter: T,
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
    type EntityIdType = EntityId;

    fn new(value: &Self::EntityIdType) -> Self {
        Self { value: *value }
    }
}

impl EntityIdSubmessageElementAttributes for EntityIdSubmessageElementPsm {
    type EntityIdType = EntityId;

    fn value(&self) -> &Self::EntityIdType {
        &self.value
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
    type SequenceNumberType = SequenceNumber;

    fn new(value: &Self::SequenceNumberType) -> Self {
        Self { value: *value }
    }
}

impl SequenceNumberSubmessageElementAttributes for SequenceNumberSubmessageElementPsm {
    fn value(&self) -> &SequenceNumber {
        &self.value
    }

    type SequenceNumberType = SequenceNumber;
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
    type TimeType = Time;

    fn value(&self) -> &Self::TimeType {
        &self.value
    }
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataSubmessageElementPsm<'a> {
    pub value: &'a [u8],
}

impl<'a> SerializedDataSubmessageElementAttributes for SerializedDataSubmessageElementPsm<'a> {
    type SerializedDataType = [u8];
    fn value(&self) -> &Self::SerializedDataType {
        &self.value
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementPsm {
    pub parameter: Vec<Parameter<Vec<u8>>>,
}

impl ParameterListSubmessageElementAttributes for ParameterListSubmessageElementPsm {
    type ParameterListType = [Parameter<Vec<u8>>];
    fn parameter(&self) -> &Self::ParameterListType {
        self.parameter.as_ref()
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementWritePsm<'a> {
    pub parameter: &'a [Parameter<Vec<u8>>],
}

#[derive(Debug, PartialEq)]
pub struct SequenceNumberSetSubmessageElementPsm {
    pub base: SequenceNumber,
    pub set: Vec<SequenceNumber>,
}

impl SequenceNumberSetSubmessageElementConstructor for SequenceNumberSetSubmessageElementPsm {
    type SequenceNumberType = SequenceNumber;
    type SequenceNumberSetType = [SequenceNumber];

    fn new(base: &Self::SequenceNumberType, set: &Self::SequenceNumberSetType) -> Self {
        Self {
            base: *base,
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
    type CountType = Count;

    fn new(_value: &Self::CountType) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct GroupDigestSubmessageElementPsm {
    pub value: GroupDigest,
}
