use std::{future::Future, pin::Pin};

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

impl
    dust_dds::domain::domain_participant_listener::DomainParticipantListener<
        dust_dds::runtime::StdRuntime,
    > for DomainParticipantListener
{
    fn on_data_available(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((),);
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_data_available", args, None)
                    .unwrap();
            })
        })
    }

    fn on_sample_rejected(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), SampleRejectedStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_sample_rejected", args, None)
                    .unwrap();
            })
        })
    }

    fn on_liveliness_changed(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), LivelinessChangedStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_liveliness_changed", args, None)
                    .unwrap();
            })
        })
    }

    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), RequestedDeadlineMissedStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_requested_deadline_missed", args, None)
                    .unwrap();
            })
        })
    }

    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), RequestedIncompatibleQosStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_requested_incompatible_qos", args, None)
                    .unwrap();
            })
        })
    }

    fn on_subscription_matched(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), SubscriptionMatchedStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_subscription_matched", args, None)
                    .unwrap();
            })
        })
    }

    fn on_sample_lost(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), SampleLostStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_sample_lost", args, None)
                    .unwrap();
            })
        })
    }

    fn on_inconsistent_topic(
        &mut self,
        the_topic: dust_dds::dds_async::topic::TopicAsync<dust_dds::runtime::StdRuntime>,
        status: dust_dds::infrastructure::status::InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
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
        })
    }

    fn on_liveliness_lost(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), LivelinessLostStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_liveliness_lost", args, None)
                    .unwrap();
            })
        })
    }

    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), OfferedDeadlineMissedStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_offered_deadline_missed", args, None)
                    .unwrap();
            })
        })
    }

    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), OfferedIncompatibleQosStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_offered_incompatible_qos", args, None)
                    .unwrap();
            })
        })
    }

    fn on_publication_matched(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<
            dust_dds::runtime::StdRuntime,
            (),
        >,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let args = ((), PublicationMatchedStatus::from(status));
            Python::with_gil(|py| {
                self.0
                    .bind(py)
                    .call_method("on_publication_matched", args, None)
                    .unwrap();
            })
        })
    }
}
