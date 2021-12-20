use rust_rtps_pim::{
    messages::submessage_elements::{
        EntityIdSubmessageElementAttributes, Parameter, ParameterListSubmessageElementAttributes,
        SequenceNumberSubmessageElementAttributes, SerializedDataSubmessageElementAttributes,
    },
    structure::types::{EntityId, SequenceNumber},
};

pub struct EntityIdSubmessageElement;

impl EntityIdSubmessageElementAttributes for EntityIdSubmessageElement {
    fn value(&self) -> &EntityId {
        todo!()
    }
}

pub struct SequenceNumberSubmessageElement;

impl SequenceNumberSubmessageElementAttributes for SequenceNumberSubmessageElement {
    fn value(&self) -> &SequenceNumber {
        todo!()
    }
}

pub struct SerializedDataSubmessageElement;

impl SerializedDataSubmessageElementAttributes for SerializedDataSubmessageElement {
    fn value(&self) -> &[u8] {
        todo!()
    }
}

pub struct ParameterListSubmessageElement;

impl ParameterListSubmessageElementAttributes for ParameterListSubmessageElement {
    fn parameter(&self) -> &[Parameter<&[u8]>] {
        todo!()
    }
}
