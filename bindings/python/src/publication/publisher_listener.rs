use pyo3::prelude::*;

use crate::infrastructure::status::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

#[derive(Clone)]
pub struct PublisherListener(Py<PyAny>);
impl From<Py<PyAny>> for PublisherListener {
    fn from(value: Py<PyAny>) -> Self {
        Self(value)
    }
}

impl dust_dds::publication::publisher_listener::PublisherListener for PublisherListener {
    fn on_liveliness_lost(
        &mut self,
        _the_writer: &dyn dust_dds::publication::data_writer::AnyDataWriter,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) {
        let args = ((), LivelinessLostStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_liveliness_lost", args, None)
                .unwrap();
        })
    }

    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &dyn dust_dds::publication::data_writer::AnyDataWriter,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) {
        let args = ((), OfferedDeadlineMissedStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_offered_deadline_missed", args, None)
                .unwrap();
        })
    }

    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &dyn dust_dds::publication::data_writer::AnyDataWriter,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) {
        let args = ((), OfferedIncompatibleQosStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_offered_incompatible_qos", args, None)
                .unwrap();
        })
    }

    fn on_publication_matched(
        &mut self,
        _the_writer: &dyn dust_dds::publication::data_writer::AnyDataWriter,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) {
        let args = ((), PublicationMatchedStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_publication_matched", args, None)
                .unwrap();
        })
    }
}
