use wasm_bindgen::prelude::*;

use super::qos_policy::{EntityFactoryQosPolicy, UserDataQosPolicy};

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DomainParticipantFactoryQos {
    pub(crate) inner: dust_dds::infrastructure::qos::DomainParticipantFactoryQos,
}

#[wasm_bindgen]
impl DomainParticipantFactoryQos {
    #[wasm_bindgen(constructor)]
    pub fn new(entity_factory: Option<EntityFactoryQosPolicy>) -> Self {
        let entity_factory = entity_factory.map(Into::into).unwrap_or_default();
        Self {
            inner: dust_dds::infrastructure::qos::DomainParticipantFactoryQos { entity_factory },
        }
    }

    #[wasm_bindgen(getter, js_name = entityFactory)]
    pub fn entity_factory(&self) -> EntityFactoryQosPolicy {
        self.inner.entity_factory.clone().into()
    }

    #[wasm_bindgen(setter, js_name = setEntityFactory)]
    pub fn set_entity_factory(&mut self, entity_factory: EntityFactoryQosPolicy) {
        self.inner.entity_factory = entity_factory.into();
    }
}

impl From<dust_dds::infrastructure::qos::DomainParticipantFactoryQos>
    for DomainParticipantFactoryQos
{
    fn from(inner: dust_dds::infrastructure::qos::DomainParticipantFactoryQos) -> Self {
        Self { inner }
    }
}

impl From<DomainParticipantFactoryQos>
    for dust_dds::infrastructure::qos::DomainParticipantFactoryQos
{
    fn from(value: DomainParticipantFactoryQos) -> Self {
        value.inner
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DomainParticipantQos {
    pub(crate) inner: dust_dds::infrastructure::qos::DomainParticipantQos,
}

#[wasm_bindgen]
impl DomainParticipantQos {
    #[wasm_bindgen(constructor)]
    pub fn new(
        user_data: Option<UserDataQosPolicy>,
        entity_factory: Option<EntityFactoryQosPolicy>,
    ) -> Self {
        let user_data = user_data.map(Into::into).unwrap_or_default();
        let entity_factory = entity_factory.map(Into::into).unwrap_or_default();
        Self {
            inner: dust_dds::infrastructure::qos::DomainParticipantQos {
                user_data,
                entity_factory,
                property: Default::default(),
            },
        }
    }

    #[wasm_bindgen(getter, js_name = userData)]
    pub fn user_data(&self) -> UserDataQosPolicy {
        self.inner.user_data.clone().into()
    }

    #[wasm_bindgen(setter, js_name = setUserData)]
    pub fn set_user_data(&mut self, user_data: UserDataQosPolicy) {
        self.inner.user_data = user_data.into();
    }

    #[wasm_bindgen(getter, js_name = entityFactory)]
    pub fn entity_factory(&self) -> EntityFactoryQosPolicy {
        self.inner.entity_factory.clone().into()
    }

    #[wasm_bindgen(setter, js_name = setEntityFactory)]
    pub fn set_entity_factory(&mut self, entity_factory: EntityFactoryQosPolicy) {
        self.inner.entity_factory = entity_factory.into();
    }
}

impl From<dust_dds::infrastructure::qos::DomainParticipantQos> for DomainParticipantQos {
    fn from(inner: dust_dds::infrastructure::qos::DomainParticipantQos) -> Self {
        Self { inner }
    }
}

impl From<DomainParticipantQos> for dust_dds::infrastructure::qos::DomainParticipantQos {
    fn from(value: DomainParticipantQos) -> Self {
        value.inner
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PublisherQos {
    pub(crate) inner: dust_dds::infrastructure::qos::PublisherQos,
}

#[wasm_bindgen]
impl PublisherQos {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl From<dust_dds::infrastructure::qos::PublisherQos> for PublisherQos {
    fn from(inner: dust_dds::infrastructure::qos::PublisherQos) -> Self {
        Self { inner }
    }
}

impl From<PublisherQos> for dust_dds::infrastructure::qos::PublisherQos {
    fn from(value: PublisherQos) -> Self {
        value.inner
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SubscriberQos {
    pub(crate) inner: dust_dds::infrastructure::qos::SubscriberQos,
}

#[wasm_bindgen]
impl SubscriberQos {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl From<dust_dds::infrastructure::qos::SubscriberQos> for SubscriberQos {
    fn from(inner: dust_dds::infrastructure::qos::SubscriberQos) -> Self {
        Self { inner }
    }
}

impl From<SubscriberQos> for dust_dds::infrastructure::qos::SubscriberQos {
    fn from(value: SubscriberQos) -> Self {
        value.inner
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TopicQos {
    pub(crate) inner: dust_dds::infrastructure::qos::TopicQos,
}

#[wasm_bindgen]
impl TopicQos {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl From<dust_dds::infrastructure::qos::TopicQos> for TopicQos {
    fn from(inner: dust_dds::infrastructure::qos::TopicQos) -> Self {
        Self { inner }
    }
}

impl From<TopicQos> for dust_dds::infrastructure::qos::TopicQos {
    fn from(value: TopicQos) -> Self {
        value.inner
    }
}
