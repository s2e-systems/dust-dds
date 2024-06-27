use dust_dds::serialized_payload::cdr::serializer::CdrSerializer;
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyDict, PyString},
};

use crate::{
    topic_definition::type_support::{PythonDdsData, TypeKind},
    xtypes::{
        cdr_serializer::ClassicCdrSerializer,
        endianness::{CdrEndianness, CDR_LE, REPRESENTATION_OPTIONS},
    },
};

#[pyclass]
pub struct DataWriter(dust_dds::publication::data_writer::DataWriter<PythonDdsData>);

impl From<dust_dds::publication::data_writer::DataWriter<PythonDdsData>> for DataWriter {
    fn from(value: dust_dds::publication::data_writer::DataWriter<PythonDdsData>) -> Self {
        Self(value)
    }
}

fn serialize_data(
    py: Python<'_>,
    data: Py<PyAny>,
    serializer: &mut ClassicCdrSerializer<&mut Vec<u8>>,
) -> PyResult<()> {
    let annotations = data
        .getattr(py, "__class__")
        .and_then(|c| c.getattr(py, "__annotations__"))?;
    let annotation_dict = annotations
        .downcast_bound::<PyDict>(py)
        .map_err(|e| PyErr::from(e))?;
    for (member_name, member_type) in annotation_dict {
        let attribute = data.getattr(py, member_name.downcast::<PyString>()?)?;
        let member_type_kind = member_type.extract::<TypeKind>()?;
        match member_type_kind {
            TypeKind::boolean => serializer.serialize_bool(attribute.extract(py)?),
            TypeKind::byte => serializer.serialize_u8(attribute.extract(py)?),
            TypeKind::char8 => serializer.serialize_char(attribute.extract(py)?),
            TypeKind::char16 => serializer.serialize_char(attribute.extract(py)?),
            TypeKind::int8 => serializer.serialize_i8(attribute.extract(py)?),
            TypeKind::uint8 => serializer.serialize_u8(attribute.extract(py)?),
            TypeKind::int16 => serializer.serialize_i16(attribute.extract(py)?),
            TypeKind::uint16 => serializer.serialize_u16(attribute.extract(py)?),
            TypeKind::int32 => serializer.serialize_i32(attribute.extract(py)?),
            TypeKind::uint32 => serializer.serialize_u32(attribute.extract(py)?),
            TypeKind::int64 => serializer.serialize_i64(attribute.extract(py)?),
            TypeKind::uint64 => serializer.serialize_u64(attribute.extract(py)?),
            TypeKind::float32 => serializer.serialize_f32(attribute.extract(py)?),
            TypeKind::float64 => serializer.serialize_f64(attribute.extract(py)?),
            TypeKind::float128 => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "float128 type not yet supported",
            )),
        }?;
    }

    Ok(())
}

#[pymethods]
impl DataWriter {
    pub fn write(&self, data: Py<PyAny>) -> PyResult<()> {
        let mut buffer = Vec::new();
        buffer.extend(&CDR_LE);
        buffer.extend(&REPRESENTATION_OPTIONS);
        let mut serializer = ClassicCdrSerializer::new(&mut buffer, CdrEndianness::LittleEndian);
        let _ = Python::with_gil(|py| serialize_data(py, data, &mut serializer))?;

        let dds_data = PythonDdsData {
            data: buffer,
            key: Vec::new(),
        };
        match self.0.write(&dds_data, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
