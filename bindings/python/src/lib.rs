mod domain;
mod infrastructure;

use domain::{
    domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
};
use infrastructure::qos_policy::UserDataQosPolicy;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn dust_dds_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let infrastructure_module = PyModule::new_bound(m.py(), "infrastructure")?;

    let qos_policy_module = PyModule::new_bound(m.py(), "qos_policy")?;
    qos_policy_module.add_class::<UserDataQosPolicy>()?;

    infrastructure_module.add_submodule(&qos_policy_module)?;

    let participant_module = PyModule::new_bound(m.py(), "participant")?;
    participant_module.add_class::<DomainParticipantFactory>()?;
    participant_module.add_class::<DomainParticipant>()?;

    m.add_submodule(&infrastructure_module)?;
    m.add_submodule(&participant_module)?;
    Ok(())
}
