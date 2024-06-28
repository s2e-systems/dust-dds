use pyo3::prelude::*;

use crate::{
    infrastructure::{error::into_pyerr, instance::InstanceHandle},
    topic_definition::type_support::PythonDdsData,
};

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

    pub fn write(&self, data: Py<PyAny>, handle: Option<InstanceHandle>) -> PyResult<()> {
        self.0
            .write(
                &PythonDdsData::from_py_object(data)?,
                handle.map(|h| h.into()),
            )
            .map_err(into_pyerr)
    }
}
