use dust_dds::{
    infrastructure::{error::DdsResult, instance::InstanceHandle},
    serialized_payload::cdr::{deserializer::CdrDeserializer, serializer::CdrSerializer},
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
};
use pyo3::{
    prelude::*,
    types::{PyDict, PyString, PyType},
};

use crate::xtypes::{
    cdr_deserializer::ClassicCdrDeserializer,
    cdr_serializer::ClassicCdrSerializer,
    endianness::{self, CdrEndianness, CDR_LE, REPRESENTATION_OPTIONS},
};

#[allow(non_camel_case_types)]
#[pyclass]
#[derive(Clone)]
pub enum TypeKind {
    boolean,
    byte,
    char8,
    char16,
    int8,
    uint8,
    int16,
    uint16,
    int32,
    uint32,
    int64,
    uint64,
    float32,
    float64,
    float128,
}

pub struct PythonTypeRepresentation(Py<PyAny>);

impl From<Py<PyAny>> for PythonTypeRepresentation {
    fn from(value: Py<PyAny>) -> Self {
        Self(value)
    }
}

impl dust_dds::topic_definition::type_support::DynamicTypeInterface for PythonTypeRepresentation {
    fn has_key(&self) -> bool {
        true
    }

    fn get_serialized_key_from_serialized_foo(&self, _serialized_foo: &[u8]) -> DdsResult<Vec<u8>> {
        todo!()
    }

    fn instance_handle_from_serialized_foo(
        &self,
        _serialized_foo: &[u8],
    ) -> DdsResult<dust_dds::infrastructure::instance::InstanceHandle> {
        Ok(InstanceHandle::default())
    }

    fn instance_handle_from_serialized_key(
        &self,
        _serialized_key: &[u8],
    ) -> DdsResult<dust_dds::infrastructure::instance::InstanceHandle> {
        todo!()
    }

    fn xml_type(&self) -> String {
        String::new()
    }

    fn user_data(&self) -> Option<&dyn std::any::Any> {
        Some(&self.0)
    }
}

#[pyclass]
pub struct PythonDdsData {
    pub data: Vec<u8>,
    pub key: Vec<u8>,
}

impl PythonDdsData {
    pub fn from_py_object(py_object: Py<PyAny>) -> PyResult<Self> {
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

        let mut buffer = Vec::new();
        buffer.extend(&CDR_LE);
        buffer.extend(&REPRESENTATION_OPTIONS);
        let mut serializer = ClassicCdrSerializer::new(&mut buffer, CdrEndianness::LittleEndian);
        let _ = Python::with_gil(|py| serialize_data(py, py_object, &mut serializer))?;

        Ok(PythonDdsData {
            data: buffer,
            key: Vec::new(),
        })
    }

    pub fn into_py_object(&self, type_: &Py<PyAny>) -> PyResult<Py<PyAny>> {
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

        let (header, body) = self.data.split_at(4);
        let endianness = match [header[0], header[1]] {
            endianness::CDR_LE => endianness::CdrEndianness::LittleEndian,
            endianness::CDR_BE => endianness::CdrEndianness::BigEndian,
            _ => panic!("Unknown endianness"),
        };
        let mut deserializer = ClassicCdrDeserializer::new(body, endianness);
        Python::with_gil(|py| {
            let type_ = type_.extract(py)?;
            deserialize_data(py, type_, &mut deserializer)
        })
    }
}

impl DdsHasKey for PythonDdsData {
    const HAS_KEY: bool = true;
}

impl DdsKey for PythonDdsData {
    type Key = Vec<u8>;

    fn get_key(&self) -> DdsResult<Self::Key> {
        Ok(self.key.clone())
    }

    fn get_key_from_serialized_data(_serialized_foo: &[u8]) -> DdsResult<Self::Key> {
        todo!()
    }
}

impl DdsSerialize for PythonDdsData {
    fn serialize_data(&self) -> DdsResult<Vec<u8>> {
        Ok(self.data.clone())
    }
}

impl<'de> DdsDeserialize<'de> for PythonDdsData {
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        Ok(Self {
            data: serialized_data.to_vec(),
            key: Vec::new(),
        })
    }
}

impl DdsTypeXml for PythonDdsData {
    fn get_type_xml() -> Option<String> {
        todo!()
    }
}
