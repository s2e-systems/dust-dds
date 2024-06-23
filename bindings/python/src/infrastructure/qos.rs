use pyo3::prelude::*;

use super::qos_policy::{EntityFactoryQosPolicy, UserDataQosPolicy};

#[pyclass]
pub struct DomainParticipantQos(dust_dds::infrastructure::qos::DomainParticipantQos);

impl From<DomainParticipantQos> for dust_dds::infrastructure::qos::DomainParticipantQos {
    fn from(value: DomainParticipantQos) -> Self {
        value.0
    }
}

#[pymethods]
impl DomainParticipantQos {
    #[new]
    #[pyo3(signature = (user_data= UserDataQosPolicy::default(), entity_factory = EntityFactoryQosPolicy::default()))]
    pub fn new(user_data: UserDataQosPolicy, entity_factory: EntityFactoryQosPolicy) -> Self {
        Self(dust_dds::infrastructure::qos::DomainParticipantQos {
            user_data: user_data.clone().into(),
            entity_factory: entity_factory.into(),
        })
    }
}
