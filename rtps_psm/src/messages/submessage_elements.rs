use rust_rtps_pim::{
    messages::submessage_elements::{
        EntityIdSubmessageElementAttributes, Parameter, ParameterListSubmessageElementAttributes,
        SequenceNumberSubmessageElementAttributes, SerializedDataSubmessageElementAttributes,
    },
    structure::types::{EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq)]
pub struct EntityIdSubmessageElementPsm {
    pub value: EntityId,
}

impl EntityIdSubmessageElementAttributes for EntityIdSubmessageElementPsm {
    fn value(&self) -> &EntityId {
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

impl SerializedDataSubmessageElementAttributes for SerializedDataSubmessageElementPsm<'_> {
    fn value(&self) -> &[u8] {
        &self.value
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementPsm<'a> {
    pub parameter: Vec<Parameter<&'a [u8]>>,
}

impl ParameterListSubmessageElementAttributes for ParameterListSubmessageElementPsm<'_> {
    fn parameter(&self) -> &[Parameter<&[u8]>] {
        self.parameter.as_ref()
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElementWritePsm {
    pub parameter: Vec<Parameter<Vec<u8>>>,
}