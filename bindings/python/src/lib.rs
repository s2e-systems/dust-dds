mod domain;
mod infrastructure;
mod publication;
mod subscription;
mod topic_definition;

use domain::{
    domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
};
use infrastructure::{
    qos::DomainParticipantQos,
    qos_policy::{EntityFactoryQosPolicy, UserDataQosPolicy},
};
use pyo3::prelude::*;

fn infrastructure_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let infrastructure_module = PyModule::new_bound(m.py(), "infrastructure")?;

    let qos_policy_module = PyModule::new_bound(m.py(), "qos_policy")?;
    qos_policy_module.add_class::<UserDataQosPolicy>()?;
    qos_policy_module.add_class::<EntityFactoryQosPolicy>()?;

    infrastructure_module.add_submodule(&qos_policy_module)?;

    let qos_module = PyModule::new_bound(m.py(), "qos")?;
    qos_module.add_class::<DomainParticipantQos>()?;
    infrastructure_module.add_submodule(&qos_module)?;

    m.add_submodule(&infrastructure_module)?;
    Ok(())
}

fn participant_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let participant_module = PyModule::new_bound(m.py(), "participant")?;
    participant_module.add_class::<DomainParticipantFactory>()?;
    participant_module.add_class::<DomainParticipant>()?;

    m.add_submodule(&participant_module)?;
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn dust_dds_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    infrastructure_module(m)?;
    participant_module(m)?;
    Ok(())
}
