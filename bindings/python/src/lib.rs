mod domain;
mod infrastructure;
mod publication;
mod subscription;
mod topic_definition;

use domain::domain_participant_factory::DomainParticipantFactory;
use infrastructure::{qos::DomainParticipantQos, qos_policy::UserDataQosPolicy};
use pyo3::prelude::*;
use topic_definition::type_support::MyDdsData;

/// Dust DDS python bindings
#[pymodule]
fn dust_dds(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DomainParticipantFactory>()?;
    m.add_class::<MyDdsData>()?;
    m.add_class::<DomainParticipantQos>()?;
    m.add_class::<UserDataQosPolicy>()?;

    Ok(())
}
