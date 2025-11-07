use pyo3::prelude::*;

use crate::{
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    topic_definition::type_support::PythonDdsData,
};

use super::data_reader::DataReader;

#[derive(Clone)]
pub struct DataReaderListener(Py<PyAny>);

impl From<Py<PyAny>> for DataReaderListener {
    fn from(value: Py<PyAny>) -> Self {
        Self(value)
    }
}

impl
    dust_dds::subscription::data_reader_listener::DataReaderListener<
        dust_dds::std_runtime::StdRuntime,
        PythonDdsData,
    > for DataReaderListener
{
    #[tracing::instrument(skip(self, the_reader))]
    async fn on_data_available(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::std_runtime::StdRuntime,
            PythonDdsData,
        >,
    ) {
        let reader = DataReader::from(the_reader);
        let args = (reader,);
        Python::attach(|py| {
            self.0
                .bind(py)
                .call_method("on_data_available", args, None)
                .unwrap();
        })
    }

    async fn on_sample_rejected(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::std_runtime::StdRuntime,
            PythonDdsData,
        >,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) {
        let args = (
            DataReader::from(the_reader),
            SampleRejectedStatus::from(status),
        );
        Python::attach(|py| {
            self.0
                .bind(py)
                .call_method("on_sample_rejected", args, None)
                .unwrap();
        })
    }

    async fn on_liveliness_changed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::std_runtime::StdRuntime,
            PythonDdsData,
        >,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) {
        let args = (
            DataReader::from(the_reader),
            LivelinessChangedStatus::from(status),
        );
        Python::attach(|py| {
            self.0
                .bind(py)
                .call_method("on_liveliness_changed", args, None)
                .unwrap();
        })
    }

    async fn on_requested_deadline_missed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::std_runtime::StdRuntime,
            PythonDdsData,
        >,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) {
        let args = (
            DataReader::from(the_reader),
            RequestedDeadlineMissedStatus::from(status),
        );
        Python::attach(|py| {
            self.0
                .bind(py)
                .call_method("on_requested_deadline_missed", args, None)
                .unwrap();
        })
    }

    async fn on_requested_incompatible_qos(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::std_runtime::StdRuntime,
            PythonDdsData,
        >,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) {
        let args = (
            DataReader::from(the_reader),
            RequestedIncompatibleQosStatus::from(status),
        );
        Python::attach(|py| {
            self.0
                .bind(py)
                .call_method("on_requested_incompatible_qos", args, None)
                .unwrap();
        })
    }

    async fn on_subscription_matched(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::std_runtime::StdRuntime,
            PythonDdsData,
        >,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
        let args = (
            DataReader::from(the_reader),
            SubscriptionMatchedStatus::from(status),
        );
        Python::attach(|py| {
            self.0
                .bind(py)
                .call_method("on_subscription_matched", args, None)
                .unwrap();
        })
    }

    async fn on_sample_lost(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<
            dust_dds::std_runtime::StdRuntime,
            PythonDdsData,
        >,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) {
        let args = (DataReader::from(the_reader), SampleLostStatus::from(status));
        Python::attach(|py| {
            self.0
                .bind(py)
                .call_method("on_sample_lost", args, None)
                .unwrap();
        })
    }
}
