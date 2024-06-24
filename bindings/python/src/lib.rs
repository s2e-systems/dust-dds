mod domain;
mod infrastructure;
mod publication;
mod subscription;
mod topic_definition;

use domain::domain_participant_factory::DomainParticipantFactory;
use pyo3::prelude::*;
use topic_definition::type_support::MyDdsData;

/// Dust DDS python bindings
#[pymodule]
fn dust_dds(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DomainParticipantFactory>()?;
    m.add_class::<MyDdsData>()?;

    Ok(())
}
