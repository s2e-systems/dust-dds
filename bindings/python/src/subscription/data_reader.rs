use dust_dds::{
    serialized_payload::cdr::deserializer::CdrDeserializer,
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
};
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyType};

use crate::{
    infrastructure::error::into_pyerr,
    topic_definition::type_support::PythonDdsData,
    xtypes::{
        cdr_deserializer::ClassicCdrDeserializer,
        endianness::{self},
    },
};

#[pyclass]
pub struct DataReader(dust_dds::subscription::data_reader::DataReader<PythonDdsData>);

impl From<dust_dds::subscription::data_reader::DataReader<PythonDdsData>> for DataReader {
    fn from(value: dust_dds::subscription::data_reader::DataReader<PythonDdsData>) -> Self {
        Self(value)
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

fn deserialize_data(
    py: Python<'_>,
    type_: Py<PyAny>,
    deserializer: &mut ClassicCdrDeserializer,
) -> PyResult<Py<PyAny>> {
    let py_type = type_.downcast_bound::<PyType>(py)?;
    let object = type_
        .bind(py)
        .call_method("__new__", (py_type,), None)?
        .unbind();

    object.setattr(py, "id", deserializer.deserialize_u32()?)?;
    object.setattr(py, "state", deserializer.deserialize_bool()?)?;
    Ok(object)
}

#[pyclass]
pub struct Sample(dust_dds::subscription::data_reader::Sample<PythonDdsData>);

#[pymethods]
impl Sample {
    pub fn get_data(&self, type_: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let python_data = self.0.data().map_err(into_pyerr)?;
        let (header, body) = python_data.data.split_at(4);
        let endianness = match [header[0], header[1]] {
            endianness::CDR_LE => endianness::CdrEndianness::LittleEndian,
            endianness::CDR_BE => endianness::CdrEndianness::BigEndian,
            _ => panic!("Unknown endianness"),
        };
        let mut deserializer = ClassicCdrDeserializer::new(body, endianness);
        Python::with_gil(|py| deserialize_data(py, type_, &mut deserializer))
    }
}
