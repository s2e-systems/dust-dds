use dust_dds::subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE};
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::topic_definition::type_support::PythonDdsData;

#[pyclass]
pub struct DataReader(dust_dds::subscription::data_reader::DataReader<PythonDdsData>);

impl From<dust_dds::subscription::data_reader::DataReader<PythonDdsData>> for DataReader {
    fn from(value: dust_dds::subscription::data_reader::DataReader<PythonDdsData>) -> Self {
        Self(value)
    }
}

#[pyclass]
pub struct Sample(dust_dds::subscription::data_reader::Sample<PythonDdsData>);

#[pymethods]
impl Sample {
    #[getter]
    pub fn get_data(&self) -> PyResult<PythonDdsData> {
        self.0
            .data()
            .map_err(|e| PyTypeError::new_err(format!("{:?}", e)))
    }
}

#[pymethods]
impl DataReader {
    pub fn read(&self, max_samples: i32) -> PyResult<Vec<Sample>> {
        match self.0.read(
            max_samples,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        ) {
            Ok(s) => Ok(s.into_iter().map(|s| Sample(s)).collect()),
            Err(dust_dds::infrastructure::error::DdsError::NoData) => Ok(Vec::new()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
