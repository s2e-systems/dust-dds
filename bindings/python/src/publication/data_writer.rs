use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::topic_definition::type_support::PythonDdsData;

#[pyclass]
pub struct DataWriter(dust_dds::publication::data_writer::DataWriter<PythonDdsData>);

impl From<dust_dds::publication::data_writer::DataWriter<PythonDdsData>> for DataWriter {
    fn from(value: dust_dds::publication::data_writer::DataWriter<PythonDdsData>) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DataWriter {
    pub fn write(&self, data: &PythonDdsData) -> PyResult<()> {
        match self.0.write(data, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
