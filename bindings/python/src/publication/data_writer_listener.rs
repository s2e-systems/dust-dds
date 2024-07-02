use pyo3::prelude::*;

use crate::{
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    topic_definition::type_support::PythonDdsData,
};

use super::data_writer::DataWriter;

#[derive(Clone)]
pub struct DataWriterListener(Py<PyAny>);
impl From<Py<PyAny>> for DataWriterListener {
    fn from(value: Py<PyAny>) -> Self {
        Self(value)
    }
}

impl dust_dds::publication::data_writer_listener::DataWriterListener<'_> for DataWriterListener {
    type Foo = PythonDdsData;

    fn on_liveliness_lost(
        &mut self,
        the_writer: dust_dds::publication::data_writer::DataWriter<Self::Foo>,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) {
        let args = (
            DataWriter::from(the_writer),
            LivelinessLostStatus::from(status),
        );
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_liveliness_lost", args, None)
                .unwrap();
        })
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: dust_dds::publication::data_writer::DataWriter<Self::Foo>,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) {
        let args = (
            DataWriter::from(the_writer),
            OfferedDeadlineMissedStatus::from(status),
        );
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_offered_deadline_missed", args, None)
                .unwrap();
        })
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: dust_dds::publication::data_writer::DataWriter<Self::Foo>,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) {
        let args = (
            DataWriter::from(the_writer),
            OfferedIncompatibleQosStatus::from(status),
        );
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_offered_incompatible_qos", args, None)
                .unwrap();
        })
    }

    fn on_publication_matched(
        &mut self,
        the_writer: dust_dds::publication::data_writer::DataWriter<Self::Foo>,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) {
        let args = (
            DataWriter::from(the_writer),
            PublicationMatchedStatus::from(status),
        );
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_publication_matched", args, None)
                .unwrap();
        })
    }
}
