use crate::domain::domain_participant::DomainParticipant;
use pyo3::prelude::*;

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct TopicDescription {
    participant: dust_dds::domain::domain_participant::DomainParticipant,
    type_name: String,
    name: String,
}

// impl From<&dyn dust_dds::topic_definition::topic_description::TopicDescription>
//     for TopicDescription
// {
//     fn from(value: &dyn dust_dds::topic_definition::topic_description::TopicDescription) -> Self {
//         Self {
//             participant: value.get_participant(),
//             type_name: value.get_type_name(),
//             name: value.get_name(),
//         }
//     }
// }

impl<T: dust_dds::topic_definition::topic_description::TopicDescription> From<T>
    for TopicDescription
{
    fn from(value: T) -> Self {
        Self {
            participant: value.get_participant(),
            type_name: value.get_type_name(),
            name: value.get_name(),
        }
    }
}

#[pymethods]
impl TopicDescription {
    pub fn get_participant(&self) -> DomainParticipant {
        self.participant.clone().into()
    }

    pub fn get_type_name(&self) -> String {
        self.type_name.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
