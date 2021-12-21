use rust_rtps_pim::{
    messages::submessage_elements::{
        EntityIdSubmessageElementAttributes, Parameter, ParameterListSubmessageElementAttributes,
        SequenceNumberSubmessageElementAttributes, SerializedDataSubmessageElementAttributes,
    },
    structure::types::{EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq)]
pub struct EntityIdSubmessageElement;

impl EntityIdSubmessageElementAttributes for EntityIdSubmessageElement {
    fn value(&self) -> &EntityId {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct SequenceNumberSubmessageElement;

impl SequenceNumberSubmessageElementAttributes for SequenceNumberSubmessageElement {
    fn value(&self) -> &SequenceNumber {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct SerializedDataSubmessageElement<'a>(pub &'a [u8]);

impl SerializedDataSubmessageElementAttributes for SerializedDataSubmessageElement<'_> {
    fn value(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListSubmessageElement;

impl ParameterListSubmessageElementAttributes for ParameterListSubmessageElement {
    fn parameter(&self) -> &[Parameter<&[u8]>] {
        todo!()
    }
}
