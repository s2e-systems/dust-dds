use pyo3::prelude::*;

#[pyclass]
pub struct DomainParticipantQos(dust_dds::infrastructure::qos::DomainParticipantQos);

impl From<DomainParticipantQos> for dust_dds::infrastructure::qos::DomainParticipantQos {
    fn from(value: DomainParticipantQos) -> Self {
        value.0
    }
}
