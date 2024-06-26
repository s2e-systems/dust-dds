use pyo3::prelude::*;

use crate::topic_definition::type_support::MyDdsData;

use super::data_reader::DataReader;

#[pyclass]
#[derive(Clone)]
pub struct DataReaderListener(Py<PyAny>);

impl From<Py<PyAny>> for DataReaderListener {
    fn from(value: Py<PyAny>) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DataReaderListener {
    #[new]
    pub fn new(listener: Py<PyAny>) -> Self {
        Self(listener)
    }
}

impl dust_dds::subscription::data_reader_listener::DataReaderListener<'_> for DataReaderListener {
    type Foo = MyDdsData;

    fn on_data_available(
        &mut self,
        the_reader: dust_dds::subscription::data_reader::DataReader<Self::Foo>,
    ) {
        let reader = DataReader::from(the_reader);
        let args = (reader,);
        Python::with_gil(|py| {
            self.0
                .bind(py)
                .call_method("on_data_available", args, None)
                .unwrap();
        })
    }

    fn on_sample_rejected(
        &mut self,
        _the_reader: dust_dds::subscription::data_reader::DataReader<Self::Foo>,
        _status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) {
    }

    fn on_liveliness_changed(
        &mut self,
        _the_reader: dust_dds::subscription::data_reader::DataReader<Self::Foo>,
        _status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) {
    }

    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: dust_dds::subscription::data_reader::DataReader<Self::Foo>,
        _status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) {
    }

    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: dust_dds::subscription::data_reader::DataReader<Self::Foo>,
        _status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) {
    }

    fn on_subscription_matched(
        &mut self,
        _the_reader: dust_dds::subscription::data_reader::DataReader<Self::Foo>,
        _status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
    }

    fn on_sample_lost(
        &mut self,
        _the_reader: dust_dds::subscription::data_reader::DataReader<Self::Foo>,
        _status: dust_dds::infrastructure::status::SampleLostStatus,
    ) {
    }
}
