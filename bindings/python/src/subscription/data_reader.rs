use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError},
    prelude::*,
};

use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    infrastructure::{
        condition::StatusCondition,
        error::into_pyerr,
        instance::InstanceHandle,
        qos::DataReaderQos,
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::Duration,
    },
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::{topic::Topic, type_support::PythonDdsData},
};

use super::{
    data_reader_listener::DataReaderListener,
    sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    subscriber::Subscriber,
};

#[pyclass]
pub struct DataReader(dust_dds::subscription::data_reader::DataReader<PythonDdsData>);

impl From<dust_dds::subscription::data_reader::DataReader<PythonDdsData>> for DataReader {
    fn from(value: dust_dds::subscription::data_reader::DataReader<PythonDdsData>) -> Self {
        Self(value)
    }
}

impl AsRef<dust_dds::subscription::data_reader::DataReader<PythonDdsData>> for DataReader {
    fn as_ref(&self) -> &dust_dds::subscription::data_reader::DataReader<PythonDdsData> {
        &self.0
    }
}

impl DataReader {
    fn get_data_type(&self) -> PyResult<Py<PyAny>> {
        let type_support = self
            .0
            .get_topicdescription()
            .get_type_support()
            .map_err(into_pyerr)?;
        type_support
            .user_data()
            .ok_or(PyRuntimeError::new_err("Type missing user data"))?
            .downcast_ref::<Py<PyAny>>()
            .ok_or(PyTypeError::new_err(
                "Type support user data should be of PyAny type",
            ))
            .cloned()
    }
}

#[pymethods]
impl DataReader {
    #[pyo3(signature = (
        max_samples,
        sample_states=ANY_SAMPLE_STATE.to_vec(),
        view_states=ANY_VIEW_STATE.to_vec(),
        instance_states=ANY_INSTANCE_STATE.to_vec(),
    ))]
    pub fn read(
        &self,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> PyResult<Vec<Sample>> {
        let type_ = self.get_data_type()?;
        let sample_states: Vec<_> = sample_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::SampleStateKind::from)
            .collect();
        let view_states: Vec<_> = view_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::ViewStateKind::from)
            .collect();
        let instance_states: Vec<_> = instance_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::InstanceStateKind::from)
            .collect();
        match self
            .0
            .read(max_samples, &sample_states, &view_states, &instance_states)
        {
            Ok(s) => Ok(s
                .into_iter()
                .map(|s| Sample {
                    sample: s,
                    type_: type_.clone(),
                })
                .collect()),
            Err(dust_dds::infrastructure::error::DdsError::NoData) => Ok(Vec::new()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    #[pyo3(signature = (
        max_samples,
        sample_states=ANY_SAMPLE_STATE.to_vec(),
        view_states=ANY_VIEW_STATE.to_vec(),
        instance_states=ANY_INSTANCE_STATE.to_vec(),
    ))]
    pub fn take(
        &self,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> PyResult<Vec<Sample>> {
        let type_ = self.get_data_type()?;
        let sample_states: Vec<_> = sample_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::SampleStateKind::from)
            .collect();
        let view_states: Vec<_> = view_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::ViewStateKind::from)
            .collect();
        let instance_states: Vec<_> = instance_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::InstanceStateKind::from)
            .collect();
        match self
            .0
            .take(max_samples, &sample_states, &view_states, &instance_states)
        {
            Ok(s) => Ok(s
                .into_iter()
                .map(|s| Sample {
                    sample: s,
                    type_: type_.clone(),
                })
                .collect()),
            Err(dust_dds::infrastructure::error::DdsError::NoData) => Ok(Vec::new()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    pub fn read_next_sample(&self) -> PyResult<Sample> {
        let type_ = self.get_data_type()?;
        match self.0.read_next_sample() {
            Ok(s) => Ok(Sample {
                sample: s,
                type_: type_.clone(),
            }),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    pub fn take_next_sample(&self) -> PyResult<Sample> {
        let type_ = self.get_data_type()?;
        match self.0.take_next_sample() {
            Ok(s) => Ok(Sample {
                sample: s,
                type_: type_.clone(),
            }),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    #[pyo3(signature = (
        max_samples,
        a_handle,
        sample_states=ANY_SAMPLE_STATE.to_vec(),
        view_states=ANY_VIEW_STATE.to_vec(),
        instance_states=ANY_INSTANCE_STATE.to_vec(),
    ))]
    pub fn read_instance(
        &self,
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> PyResult<Vec<Sample>> {
        let type_ = self.get_data_type()?;
        let sample_states: Vec<_> = sample_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::SampleStateKind::from)
            .collect();
        let view_states: Vec<_> = view_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::ViewStateKind::from)
            .collect();
        let instance_states: Vec<_> = instance_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::InstanceStateKind::from)
            .collect();
        match self.0.read_instance(
            max_samples,
            a_handle.into(),
            &sample_states,
            &view_states,
            &instance_states,
        ) {
            Ok(s) => Ok(s
                .into_iter()
                .map(|s| Sample {
                    sample: s,
                    type_: type_.clone(),
                })
                .collect()),
            Err(dust_dds::infrastructure::error::DdsError::NoData) => Ok(Vec::new()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    #[pyo3(signature = (
        max_samples,
        a_handle,
        sample_states=ANY_SAMPLE_STATE.to_vec(),
        view_states=ANY_VIEW_STATE.to_vec(),
        instance_states=ANY_INSTANCE_STATE.to_vec(),
    ))]
    pub fn take_instance(
        &self,
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> PyResult<Vec<Sample>> {
        let type_ = self.get_data_type()?;
        let sample_states: Vec<_> = sample_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::SampleStateKind::from)
            .collect();
        let view_states: Vec<_> = view_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::ViewStateKind::from)
            .collect();
        let instance_states: Vec<_> = instance_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::InstanceStateKind::from)
            .collect();
        match self.0.take_instance(
            max_samples,
            a_handle.into(),
            &sample_states,
            &view_states,
            &instance_states,
        ) {
            Ok(s) => Ok(s
                .into_iter()
                .map(|s| Sample {
                    sample: s,
                    type_: type_.clone(),
                })
                .collect()),
            Err(dust_dds::infrastructure::error::DdsError::NoData) => Ok(Vec::new()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    #[pyo3(signature = (
        max_samples,
        previous_handle,
        sample_states=ANY_SAMPLE_STATE.to_vec(),
        view_states=ANY_VIEW_STATE.to_vec(),
        instance_states=ANY_INSTANCE_STATE.to_vec(),
    ))]
    pub fn read_next_instance(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> PyResult<Vec<Sample>> {
        let type_ = self.get_data_type()?;
        let sample_states: Vec<_> = sample_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::SampleStateKind::from)
            .collect();
        let view_states: Vec<_> = view_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::ViewStateKind::from)
            .collect();
        let instance_states: Vec<_> = instance_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::InstanceStateKind::from)
            .collect();
        match self.0.read_next_instance(
            max_samples,
            previous_handle.map(|x| x.into()),
            &sample_states,
            &view_states,
            &instance_states,
        ) {
            Ok(s) => Ok(s
                .into_iter()
                .map(|s| Sample {
                    sample: s,
                    type_: type_.clone(),
                })
                .collect()),
            Err(dust_dds::infrastructure::error::DdsError::NoData) => Ok(Vec::new()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    #[pyo3(signature = (
        max_samples,
        previous_handle,
        sample_states=ANY_SAMPLE_STATE.to_vec(),
        view_states=ANY_VIEW_STATE.to_vec(),
        instance_states=ANY_INSTANCE_STATE.to_vec(),
    ))]
    pub fn take_next_instance(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> PyResult<Vec<Sample>> {
        let type_ = self.get_data_type()?;
        let sample_states: Vec<_> = sample_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::SampleStateKind::from)
            .collect();
        let view_states: Vec<_> = view_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::ViewStateKind::from)
            .collect();
        let instance_states: Vec<_> = instance_states
            .into_iter()
            .map(dust_dds::subscription::sample_info::InstanceStateKind::from)
            .collect();
        match self.0.take_next_instance(
            max_samples,
            previous_handle.map(|x| x.into()),
            &sample_states,
            &view_states,
            &instance_states,
        ) {
            Ok(s) => Ok(s
                .into_iter()
                .map(|s| Sample {
                    sample: s,
                    type_: type_.clone(),
                })
                .collect()),
            Err(dust_dds::infrastructure::error::DdsError::NoData) => Ok(Vec::new()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }

    pub fn get_key_value(&self, _key_holder: Py<PyAny>, _handle: InstanceHandle) -> PyResult<()> {
        unimplemented!()
    }

    pub fn lookup_instance(&self, _instance: Py<PyAny>) -> PyResult<Option<InstanceHandle>> {
        unimplemented!()
    }

    pub fn get_liveliness_changed_status(&self) -> PyResult<LivelinessChangedStatus> {
        Ok(self
            .0
            .get_liveliness_changed_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_requested_deadline_missed_status(&self) -> PyResult<RequestedDeadlineMissedStatus> {
        Ok(self
            .0
            .get_requested_deadline_missed_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_requested_incompatible_qos_status(
        &self,
    ) -> PyResult<RequestedIncompatibleQosStatus> {
        Ok(self
            .0
            .get_requested_incompatible_qos_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_sample_lost_status(&self) -> PyResult<SampleLostStatus> {
        Ok(self.0.get_sample_lost_status().map_err(into_pyerr)?.into())
    }

    pub fn get_sample_rejected_status(&self) -> PyResult<SampleRejectedStatus> {
        Ok(self
            .0
            .get_sample_rejected_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_subscription_matched_status(&self) -> PyResult<SubscriptionMatchedStatus> {
        Ok(self
            .0
            .get_subscription_matched_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_topicdescription(&self) -> Topic {
        self.0.get_topicdescription().into()
    }

    pub fn get_subscriber(&self) -> Subscriber {
        self.0.get_subscriber().into()
    }

    pub fn wait_for_historical_data(&self, max_wait: Duration) -> PyResult<()> {
        self.0
            .wait_for_historical_data(max_wait.into())
            .map_err(into_pyerr)
    }

    pub fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> PyResult<PublicationBuiltinTopicData> {
        Ok(self
            .0
            .get_matched_publication_data(publication_handle.into())
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_matched_publications(&self) -> PyResult<Vec<InstanceHandle>> {
        Ok(self
            .0
            .get_matched_publications()
            .map_err(into_pyerr)?
            .into_iter()
            .map(InstanceHandle::from)
            .collect())
    }

    pub fn set_qos(&self, qos: Option<DataReaderQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_qos(qos).map_err(into_pyerr)
    }

    pub fn get_qos(&self) -> PyResult<DataReaderQos> {
        match self.0.get_qos() {
            Ok(q) => Ok(q.into()),
            Err(e) => Err(into_pyerr(e)),
        }
    }

    #[pyo3(signature = (a_listener = None, mask = Vec::new()))]
    pub fn set_listener(
        &self,
        a_listener: Option<Py<PyAny>>,
        mask: Vec<StatusKind>,
    ) -> PyResult<()> {
        let listener: Option<
            Box<
                dyn dust_dds::subscription::data_reader_listener::DataReaderListener<
                        Foo = PythonDdsData,
                    > + Send,
            >,
        > = match a_listener {
            Some(l) => Some(Box::new(DataReaderListener::from(l))),
            None => None,
        };
        let mask: Vec<dust_dds::infrastructure::status::StatusKind> = mask
            .into_iter()
            .map(dust_dds::infrastructure::status::StatusKind::from)
            .collect();
        self.0.set_listener(listener, &mask).map_err(into_pyerr)
    }

    pub fn get_statuscondition(&self) -> StatusCondition {
        self.0.get_statuscondition().into()
    }

    pub fn get_status_changes(&self) -> PyResult<Vec<StatusKind>> {
        Ok(self
            .0
            .get_status_changes()
            .map_err(into_pyerr)?
            .into_iter()
            .map(StatusKind::from)
            .collect())
    }

    pub fn enable(&self) -> PyResult<()> {
        self.0.enable().map_err(into_pyerr)
    }

    pub fn get_instance_handle(&self) -> PyResult<InstanceHandle> {
        Ok(self.0.get_instance_handle().map_err(into_pyerr)?.into())
    }
}

#[pyclass]
pub struct Sample {
    sample: dust_dds::subscription::data_reader::Sample<PythonDdsData>,
    type_: Py<PyAny>,
}

#[pymethods]
impl Sample {
    pub fn get_data(&self) -> PyResult<Py<PyAny>> {
        self.sample
            .data()
            .map_err(into_pyerr)?
            .into_py_object(&self.type_)
    }

    pub fn get_sample_info(&self) -> SampleInfo {
        self.sample.sample_info().into()
    }
}
