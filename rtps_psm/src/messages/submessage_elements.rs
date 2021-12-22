use rust_rtps_pim::{
    messages::submessage_elements::{
        EntityIdSubmessageElementAttributes, EntityIdSubmessageElementConstructor, Parameter,
        ParameterListSubmessageElementAttributes, SequenceNumberSubmessageElementAttributes,
        SerializedDataSubmessageElementAttributes,
    },
    structure::types::{EntityId, SequenceNumber},
};

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
