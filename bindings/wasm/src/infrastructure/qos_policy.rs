use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityFactoryQosPolicy {
    pub autoenable_created_entities: bool,
}

#[wasm_bindgen]
impl EntityFactoryQosPolicy {
    #[wasm_bindgen(constructor)]
    pub fn new(autoenable_created_entities: Option<bool>) -> Self {
        Self {
            autoenable_created_entities: autoenable_created_entities.unwrap_or(true),
        }
    }
}

impl Default for EntityFactoryQosPolicy {
    fn default() -> Self {
        Self {
            autoenable_created_entities: true,
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy> for EntityFactoryQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy) -> Self {
        Self {
            autoenable_created_entities: value.autoenable_created_entities,
        }
    }
}

impl From<EntityFactoryQosPolicy> for dust_dds::infrastructure::qos_policy::EntityFactoryQosPolicy {
    fn from(value: EntityFactoryQosPolicy) -> Self {
        Self {
            autoenable_created_entities: value.autoenable_created_entities,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct UserDataQosPolicy {
    value: Vec<u8>,
}

#[wasm_bindgen]
impl UserDataQosPolicy {
    #[wasm_bindgen(constructor)]
    pub fn new(value: Option<Vec<u8>>) -> Self {
        Self {
            value: value.unwrap_or_default(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> Vec<u8> {
        self.value.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_value(&mut self, value: Vec<u8>) {
        self.value = value;
    }
}

impl From<dust_dds::infrastructure::qos_policy::UserDataQosPolicy> for UserDataQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::UserDataQosPolicy) -> Self {
        Self { value: value.value }
    }
}

impl From<UserDataQosPolicy> for dust_dds::infrastructure::qos_policy::UserDataQosPolicy {
    fn from(value: UserDataQosPolicy) -> Self {
        Self { value: value.value }
    }
}
