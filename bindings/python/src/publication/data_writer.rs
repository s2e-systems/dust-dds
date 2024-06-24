use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::topic_definition::type_support::MyDdsData;

#[pyclass]
pub struct DataWriter(dust_dds::publication::data_writer::DataWriter<MyDdsData>);

impl From<dust_dds::publication::data_writer::DataWriter<MyDdsData>> for DataWriter {
    fn from(value: dust_dds::publication::data_writer::DataWriter<MyDdsData>) -> Self {
        Self(value)
    }
}

#[pymethods]
impl DataWriter {
    pub fn write(&self, data: &MyDdsData) -> PyResult<()> {
        match self.0.write(data, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
