use rust_rtps_pim::{
    messages::{
        submessage_elements::{
            CountSubmessageElementConstructor, EntityIdSubmessageElementAttributes,
            EntityIdSubmessageElementConstructor, ParameterListSubmessageElementAttributes,
            SequenceNumberSetSubmessageElementConstructor,
            SequenceNumberSubmessageElementAttributes, SerializedDataSubmessageElementAttributes, ParameterListSubmessageElementConstructor,
        },
        types::{Count, ParameterId},
    },
    structure::types::{EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter<'a> {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: &'a [u8],
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
pub struct ParameterOwning {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: Vec<u8>,
}

// #[derive(Debug, PartialEq)]
// pub struct ParameterListSubmessageElement<T> {
//     pub parameter: T,
// }

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementWrite<'a> {
    pub parameter: &'a [Parameter<'a>],
}
impl<'a> ParameterListSubmessageElementConstructor<'a> for ParameterListSubmessageElementWrite<'a> {
    type ParameterListType = [Parameter<'a>];

    fn new(parameter: &'a Self::ParameterListType) -> Self where Self: 'a{
        Self {
            parameter,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementRead<'a> {
    pub parameter: Vec<Parameter<'a>>,
}
impl<'a> ParameterListSubmessageElementAttributes for ParameterListSubmessageElementRead<'a> {
    type ParameterListType = [Parameter<'a>];

    fn parameter(&self) -> &Self::ParameterListType {
        &self.parameter
    }
}

#[derive(Debug, PartialEq)]
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
pub struct SequenceNumberSubmessageElementPsm {
    pub value: SequenceNumber,
}

impl SequenceNumberSubmessageElementAttributes for SequenceNumberSubmessageElementPsm {
    fn value(&self) -> &SequenceNumber {
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
