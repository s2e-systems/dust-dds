pub mod domain;

use domain::{
    domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
};
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn dust_dds_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let participant_module = PyModule::new_bound(m.py(), "participant")?;
    participant_module.add_class::<DomainParticipantFactory>()?;
    participant_module.add_class::<DomainParticipant>()?;

    m.add_submodule(&participant_module)?;
    Ok(())
}
