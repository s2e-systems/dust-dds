use dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer;
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError},
    prelude::*,
    types::{PyDict, PyString, PyType},
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
    topic_definition::{
        topic::Topic,
        type_support::{PythonDdsData, TypeKind},
    },
    xtypes::{cdr_deserializer::ClassicCdrDeserializer, endianness},
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

    #[pyo3(signature = (max_samples, previous_handle, sample_states, view_states, instance_states))]
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

    #[pyo3(signature = (max_samples, previous_handle, sample_states, view_states, instance_states))]
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

    pub fn get_key_value(
        &self,
        _key_holder: Bound<'_, PyAny>,
        _handle: InstanceHandle,
    ) -> PyResult<()> {
        unimplemented!()
    }

    pub fn lookup_instance(
        &self,
        _instance: &Bound<'_, PyAny>,
    ) -> PyResult<Option<InstanceHandle>> {
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
        let mask: Vec<dust_dds::infrastructure::status::StatusKind> =
            mask.into_iter().map(|m| m.into()).collect();
        self.0
            .set_listener(listener, &mask)
            .map_err(|e| into_pyerr(e))
    }

    pub fn get_statuscondition(&self) -> StatusCondition {
        self.0.get_statuscondition().into()
    }

    pub fn get_status_changes(&self) -> PyResult<Vec<StatusKind>> {
        Ok(self
            .0
            .get_status_changes()
            .map_err(|e| into_pyerr(e))?
            .into_iter()
            .map(|s| s.into())
            .collect())
    }

    pub fn enable(&self) -> PyResult<()> {
        self.0.enable().map_err(|e| into_pyerr(e))
    }

    pub fn get_instance_handle(&self) -> PyResult<InstanceHandle> {
        Ok(self.0.get_instance_handle().map_err(into_pyerr)?.into())
    }
}

fn deserialize_data(
    py: Python<'_>,
    type_: Py<PyType>,
    deserializer: &mut ClassicCdrDeserializer,
) -> PyResult<Py<PyAny>> {
    let py_type = type_.bind(py);
    let object = type_
        .bind(py)
        .call_method("__new__", (py_type,), None)?
        .unbind();
    let annotations = py_type.getattr("__annotations__")?;
    let annotation_dict = annotations.downcast::<PyDict>().map_err(PyErr::from)?;
    for (member_name, member_type) in annotation_dict {
        let member_name_str = member_name.downcast::<PyString>()?;
        let member_type_kind = member_type.extract::<TypeKind>()?;
        match member_type_kind {
            TypeKind::boolean => {
                object.setattr(py, member_name_str, deserializer.deserialize_bool()?)?
            }
            TypeKind::byte => {
                object.setattr(py, member_name_str, deserializer.deserialize_u8()?)?
            }
            TypeKind::char8 => {
                object.setattr(py, member_name_str, deserializer.deserialize_char()?)?
            }
            TypeKind::char16 => {
                object.setattr(py, member_name_str, deserializer.deserialize_char()?)?
            }
            TypeKind::int8 => {
                object.setattr(py, member_name_str, deserializer.deserialize_i8()?)?
            }
            TypeKind::uint8 => {
                object.setattr(py, member_name_str, deserializer.deserialize_u8()?)?
            }
            TypeKind::int16 => {
                object.setattr(py, member_name_str, deserializer.deserialize_i16()?)?
            }
            TypeKind::uint16 => {
                object.setattr(py, member_name_str, deserializer.deserialize_u16()?)?
            }
            TypeKind::int32 => {
                object.setattr(py, member_name_str, deserializer.deserialize_i32()?)?
            }
            TypeKind::uint32 => {
                object.setattr(py, member_name_str, deserializer.deserialize_u32()?)?
            }
            TypeKind::int64 => {
                object.setattr(py, member_name_str, deserializer.deserialize_i64()?)?
            }
            TypeKind::uint64 => {
                object.setattr(py, member_name_str, deserializer.deserialize_u64()?)?
            }
            TypeKind::float32 => {
                object.setattr(py, member_name_str, deserializer.deserialize_f32()?)?
            }
            TypeKind::float64 => {
                object.setattr(py, member_name_str, deserializer.deserialize_f64()?)?
            }
            TypeKind::float128 => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "float128 type not yet supported",
            ))?,
        };
    }

    Ok(object)
}

#[pyclass]
pub struct Sample {
    sample: dust_dds::subscription::data_reader::Sample<PythonDdsData>,
    type_: Py<PyAny>,
}

#[pymethods]
impl Sample {
    #[getter]
    pub fn get_data(&self) -> PyResult<Py<PyAny>> {
        let python_data = self.sample.data().map_err(into_pyerr)?;
        let (header, body) = python_data.data.split_at(4);
        let endianness = match [header[0], header[1]] {
            endianness::CDR_LE => endianness::CdrEndianness::LittleEndian,
            endianness::CDR_BE => endianness::CdrEndianness::BigEndian,
            _ => panic!("Unknown endianness"),
        };
        let mut deserializer = ClassicCdrDeserializer::new(body, endianness);
        Python::with_gil(|py| {
            let type_ = self.type_.extract(py)?;
            deserialize_data(py, type_, &mut deserializer)
        })
    }

    #[getter]
    pub fn get_sample_info(&self) -> SampleInfo {
        self.sample.sample_info().into()
    }
}
