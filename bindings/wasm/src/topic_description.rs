use wasm_bindgen::prelude::*;

use crate::domain_participant::DomainParticipant;

#[wasm_bindgen]
#[derive(Clone)]
pub struct TopicDescription {
    participant: DomainParticipant,
    type_name: String,
    name: String,
}

impl TopicDescription {
    pub(crate) fn new(participant: DomainParticipant, type_name: String, name: String) -> Self {
        Self {
            participant,
            type_name,
            name,
        }
    }
}

#[wasm_bindgen]
impl TopicDescription {
    #[wasm_bindgen(js_name = getParticipant)]
    pub fn get_participant(&self) -> DomainParticipant {
        self.participant.clone()
    }

    #[wasm_bindgen(js_name = getTypeName)]
    pub fn get_type_name(&self) -> String {
        self.type_name.clone()
    }

    #[wasm_bindgen(js_name = getName)]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
