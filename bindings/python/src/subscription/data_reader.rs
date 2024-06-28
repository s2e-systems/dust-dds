use dust_dds::{
    serialized_payload::cdr::deserializer::CdrDeserializer,
    subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
};
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyDict, PyString, PyType},
};

use crate::{
    infrastructure::error::into_pyerr,
    topic_definition::type_support::{PythonDdsData, TypeKind},
    xtypes::{cdr_deserializer::ClassicCdrDeserializer, endianness},
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
pub struct Sample(dust_dds::subscription::data_reader::Sample<PythonDdsData>);

#[pymethods]
impl Sample {
    pub fn get_data(&self, type_: Py<PyType>) -> PyResult<Py<PyAny>> {
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
