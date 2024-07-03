use pyo3::prelude::*;

use crate::{
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    topic_definition::topic::Topic,
};

#[derive(Clone)]
pub struct DomainParticipantListener(Py<PyAny>);
impl From<Py<PyAny>> for DomainParticipantListener {
    fn from(value: Py<PyAny>) -> Self {
        Self(value)
    }
}

impl dust_dds::domain::domain_participant_listener::DomainParticipantListener
    for DomainParticipantListener
{
    fn on_data_available(
        &mut self,
        _the_reader: &dyn dust_dds::subscription::data_reader::AnyDataReader,
    ) {
        let args = ((),);
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_data_available", args, None)
                .unwrap();
        })
    }

    fn on_sample_rejected(
        &mut self,
        _the_reader: &dyn dust_dds::subscription::data_reader::AnyDataReader,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) {
        let args = ((), SampleRejectedStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_sample_rejected", args, None)
                .unwrap();
        })
    }

    fn on_liveliness_changed(
        &mut self,
        _the_reader: &dyn dust_dds::subscription::data_reader::AnyDataReader,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) {
        let args = ((), LivelinessChangedStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_liveliness_changed", args, None)
                .unwrap();
        })
    }

    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: &dyn dust_dds::subscription::data_reader::AnyDataReader,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) {
        let args = ((), RequestedDeadlineMissedStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_requested_deadline_missed", args, None)
                .unwrap();
        })
    }

    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: &dyn dust_dds::subscription::data_reader::AnyDataReader,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) {
        let args = ((), RequestedIncompatibleQosStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_requested_incompatible_qos", args, None)
                .unwrap();
        })
    }

    fn on_subscription_matched(
        &mut self,
        _the_reader: &dyn dust_dds::subscription::data_reader::AnyDataReader,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
        let args = ((), SubscriptionMatchedStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_subscription_matched", args, None)
                .unwrap();
        })
    }

    fn on_sample_lost(
        &mut self,
        _the_reader: &dyn dust_dds::subscription::data_reader::AnyDataReader,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) {
        let args = ((), SampleLostStatus::from(status));
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_sample_lost", args, None)
                .unwrap();
        })
    }

    fn on_inconsistent_topic(
        &mut self,
        the_topic: dust_dds::topic_definition::topic::Topic,
        status: dust_dds::infrastructure::status::InconsistentTopicStatus,
    ) {
        let args = (
            Topic::from(the_topic),
            InconsistentTopicStatus::from(status),
        );
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_inconsistent_topic", args, None)
                .unwrap();
        })
    }

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
