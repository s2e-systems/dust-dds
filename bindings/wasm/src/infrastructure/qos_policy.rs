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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryQosPolicyKind {
    KeepLast = 0,
    KeepAll = 1,
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryQosPolicy {
    kind: HistoryQosPolicyKind,
    depth: u32,
}

#[wasm_bindgen]
impl HistoryQosPolicy {
    #[wasm_bindgen(constructor)]
    pub fn new(kind: Option<HistoryQosPolicyKind>, depth: Option<u32>) -> Self {
        Self {
            kind: kind.unwrap_or(HistoryQosPolicyKind::KeepLast),
            depth: depth.unwrap_or(1),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> HistoryQosPolicyKind {
        self.kind
    }

    #[wasm_bindgen(setter)]
    pub fn set_kind(&mut self, kind: HistoryQosPolicyKind) {
        self.kind = kind;
    }

    #[wasm_bindgen(js_name = setKind)]
    pub fn set_kind_method(&mut self, kind: HistoryQosPolicyKind) {
        self.kind = kind;
    }

    #[wasm_bindgen(getter)]
    pub fn depth(&self) -> u32 {
        self.depth
    }

    #[wasm_bindgen(setter)]
    pub fn set_depth(&mut self, depth: u32) {
        self.depth = depth;
    }

    #[wasm_bindgen(js_name = setDepth)]
    pub fn set_depth_method(&mut self, depth: u32) {
        self.depth = depth;
    }
}

impl Default for HistoryQosPolicy {
    fn default() -> Self {
        Self {
            kind: HistoryQosPolicyKind::KeepLast,
            depth: 1,
        }
    }
}

impl From<dust_dds::infrastructure::qos_policy::HistoryQosPolicy> for HistoryQosPolicy {
    fn from(value: dust_dds::infrastructure::qos_policy::HistoryQosPolicy) -> Self {
        match value.kind {
            dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepLast(depth) => Self {
                kind: HistoryQosPolicyKind::KeepLast,
                depth,
            },
            dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepAll => Self {
                kind: HistoryQosPolicyKind::KeepAll,
                depth: 1,
            },
        }
    }
}

impl From<HistoryQosPolicy> for dust_dds::infrastructure::qos_policy::HistoryQosPolicy {
    fn from(value: HistoryQosPolicy) -> Self {
        let kind = match value.kind {
            HistoryQosPolicyKind::KeepLast => {
                dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepLast(value.depth)
            }
            HistoryQosPolicyKind::KeepAll => {
                dust_dds::infrastructure::qos_policy::HistoryQosPolicyKind::KeepAll
            }
        };
        Self { kind }
    }
}
