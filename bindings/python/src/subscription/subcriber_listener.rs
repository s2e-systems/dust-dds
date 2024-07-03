use pyo3::prelude::*;

use crate::infrastructure::status::{
    LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
    SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
};

use super::subscriber::Subscriber;

#[derive(Clone)]
pub struct SubscriberListener(Py<PyAny>);

impl From<Py<PyAny>> for SubscriberListener {
    fn from(value: Py<PyAny>) -> Self {
        Self(value)
    }
}

impl dust_dds::subscription::subscriber_listener::SubscriberListener for SubscriberListener {
    fn on_data_on_readers(
        &mut self,
        the_subscriber: dust_dds::subscription::subscriber::Subscriber,
    ) {
        let args = (Subscriber::from(the_subscriber),);
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_data_on_readers", args, None)
                .unwrap();
        })
    }

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
}
