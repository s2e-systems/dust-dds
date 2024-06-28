use pyo3::prelude::*;

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    infrastructure::{
        condition::StatusCondition,
        error::into_pyerr,
        instance::InstanceHandle,
        qos::DataWriterQos,
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::{Duration, Time},
    },
    topic_definition::{topic::Topic, type_support::PythonDdsData},
};

use super::{data_writer_listener::DataWriterListener, publisher::Publisher};

#[pyclass]
pub struct DataWriter(dust_dds::publication::data_writer::DataWriter<PythonDdsData>);

impl From<dust_dds::publication::data_writer::DataWriter<PythonDdsData>> for DataWriter {
    fn from(value: dust_dds::publication::data_writer::DataWriter<PythonDdsData>) -> Self {
        Self(value)
    }
}

impl AsRef<dust_dds::publication::data_writer::DataWriter<PythonDdsData>> for DataWriter {
    fn as_ref(&self) -> &dust_dds::publication::data_writer::DataWriter<PythonDdsData> {
        &self.0
    }
}

#[pymethods]
impl DataWriter {
    pub fn register_instance(&self, instance: Py<PyAny>) -> PyResult<Option<InstanceHandle>> {
        Ok(self
            .0
            .register_instance(&PythonDdsData::from_py_object(instance)?)
            .map_err(into_pyerr)?
            .map(InstanceHandle::from))
    }

    pub fn register_instance_w_timestamp(
        &self,
        instance: Py<PyAny>,
        timestamp: Time,
    ) -> PyResult<Option<InstanceHandle>> {
        Ok(self
            .0
            .register_instance_w_timestamp(
                &PythonDdsData::from_py_object(instance)?,
                timestamp.into(),
            )
            .map_err(into_pyerr)?
            .map(InstanceHandle::from))
    }

    pub fn unregister_instance(
        &self,
        instance: Py<PyAny>,
        handle: Option<InstanceHandle>,
    ) -> PyResult<()> {
        self.0
            .unregister_instance(
                &PythonDdsData::from_py_object(instance)?,
                handle.map(|h| h.into()),
            )
            .map_err(into_pyerr)
    }

    #[pyo3(signature = (instance, handle, timestamp))]
    pub fn unregister_instance_w_timestamp(
        &self,
        instance: Py<PyAny>,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> PyResult<()> {
        self.0
            .unregister_instance_w_timestamp(
                &PythonDdsData::from_py_object(instance)?,
                handle.map(|h| h.into()),
                timestamp.into(),
            )
            .map_err(into_pyerr)
    }

    pub fn get_key_value(&self, _key_holder: Py<PyAny>, _handle: InstanceHandle) -> PyResult<()> {
        unimplemented!()
    }

    pub fn lookup_instance(&self, instance: Py<PyAny>) -> PyResult<Option<InstanceHandle>> {
        Ok(self
            .0
            .lookup_instance(&PythonDdsData::from_py_object(instance)?)
            .map_err(into_pyerr)?
            .map(InstanceHandle::from))
    }

    pub fn write(&self, data: Py<PyAny>, handle: Option<InstanceHandle>) -> PyResult<()> {
        self.0
            .write(
                &PythonDdsData::from_py_object(data)?,
                handle.map(|h| h.into()),
            )
            .map_err(into_pyerr)
    }

    #[pyo3(signature = (data, handle, timestamp))]
    pub fn write_w_timestamp(
        &self,
        data: Py<PyAny>,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> PyResult<()> {
        self.0
            .write_w_timestamp(
                &PythonDdsData::from_py_object(data)?,
                handle.map(|h| h.into()),
                timestamp.into(),
            )
            .map_err(into_pyerr)
    }

    pub fn dispose(&self, data: Py<PyAny>, handle: Option<InstanceHandle>) -> PyResult<()> {
        self.0
            .dispose(
                &PythonDdsData::from_py_object(data)?,
                handle.map(|h| h.into()),
            )
            .map_err(into_pyerr)
    }

    #[pyo3(signature = (data, handle, timestamp))]
    pub fn dispose_w_timestamp(
        &self,
        data: Py<PyAny>,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> PyResult<()> {
        self.0
            .dispose_w_timestamp(
                &PythonDdsData::from_py_object(data)?,
                handle.map(|h| h.into()),
                timestamp.into(),
            )
            .map_err(into_pyerr)
    }

    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> PyResult<()> {
        self.0
            .wait_for_acknowledgments(max_wait.into())
            .map_err(into_pyerr)
    }

    pub fn get_liveliness_lost_status(&self) -> PyResult<LivelinessLostStatus> {
        Ok(self
            .0
            .get_liveliness_lost_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_offered_deadline_missed_status(&self) -> PyResult<OfferedDeadlineMissedStatus> {
        Ok(self
            .0
            .get_offered_deadline_missed_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_offered_incompatible_qos_status(&self) -> PyResult<OfferedIncompatibleQosStatus> {
        Ok(self
            .0
            .get_offered_incompatible_qos_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_publication_matched_status(&self) -> PyResult<PublicationMatchedStatus> {
        Ok(self
            .0
            .get_publication_matched_status()
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_topic(&self) -> Topic {
        self.0.get_topic().into()
    }

    pub fn get_publisher(&self) -> Publisher {
        self.0.get_publisher().into()
    }

    pub fn assert_liveliness(&self) -> PyResult<()> {
        self.0.assert_liveliness().map_err(into_pyerr)
    }

    pub fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> PyResult<SubscriptionBuiltinTopicData> {
        Ok(self
            .0
            .get_matched_subscription_data(subscription_handle.into())
            .map_err(into_pyerr)?
            .into())
    }

    pub fn get_matched_subscriptions(&self) -> PyResult<Vec<InstanceHandle>> {
        Ok(self
            .0
            .get_matched_subscriptions()
            .map_err(into_pyerr)?
            .into_iter()
            .map(InstanceHandle::from)
            .collect())
    }

    pub fn set_qos(&self, qos: Option<DataWriterQos>) -> PyResult<()> {
        let qos = match qos {
            Some(q) => dust_dds::infrastructure::qos::QosKind::Specific(q.into()),
            None => dust_dds::infrastructure::qos::QosKind::Default,
        };
        self.0.set_qos(qos).map_err(into_pyerr)
    }

    pub fn get_qos(&self) -> PyResult<DataWriterQos> {
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
                dyn dust_dds::publication::data_writer_listener::DataWriterListener<
                        Foo = PythonDdsData,
                    > + Send,
            >,
        > = match a_listener {
            Some(l) => Some(Box::new(DataWriterListener::from(l))),
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
