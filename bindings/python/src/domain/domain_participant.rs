use pyo3::prelude::*;

#[pyclass]
pub struct DomainParticipant(pub dust_dds::domain::domain_participant::DomainParticipant);
