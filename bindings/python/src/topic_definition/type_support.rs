use dust_dds::{
    infrastructure::{error::DdsResult, instance::InstanceHandle},
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
    xtypes::{
        deserializer::XTypesDeserializer, error::XTypesError, serializer::XTypesSerializer,
        xcdr_deserializer::Xcdr1LeDeserializer, xcdr_deserializer::Xcdr1BeDeserializer, xcdr_serializer::Xcdr1LeSerializer,
    },
};
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyBytes, PyDict, PyList, PySequence, PyString, PyTuple, PyType},
};

use crate::xtypes::endianness::{self, CDR_LE, REPRESENTATION_OPTIONS};

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

fn deserialize_into_py_object<'de, D: XTypesDeserializer<'de>>(
    py: Python<'_>,
    type_kind: TypeKind,
    deserializer: D,
) -> Result<PyObject, XTypesError> {
    match type_kind {
        TypeKind::boolean => Ok(deserializer.deserialize_boolean()?.into_py(py)),
        TypeKind::byte => Ok(deserializer.deserialize_uint8()?.into_py(py)),
        TypeKind::char8 => Ok(deserializer.deserialize_char8()?.into_py(py)),
        TypeKind::char16 => Ok(deserializer.deserialize_char8()?.into_py(py)),
        TypeKind::int8 => Ok(deserializer.deserialize_int8()?.into_py(py)),
        TypeKind::uint8 => Ok(deserializer.deserialize_uint8()?.into_py(py)),
        TypeKind::int16 => Ok(deserializer.deserialize_int16()?.into_py(py)),
        TypeKind::uint16 => Ok(deserializer.deserialize_uint16()?.into_py(py)),
        TypeKind::int32 => Ok(deserializer.deserialize_int32()?.into_py(py)),
        TypeKind::uint32 => Ok(deserializer.deserialize_uint32()?.into_py(py)),
        TypeKind::int64 => Ok(deserializer.deserialize_int64()?.into_py(py)),
        TypeKind::uint64 => Ok(deserializer.deserialize_uint64()?.into_py(py)),
        TypeKind::float32 => Ok(deserializer.deserialize_float32()?.into_py(py)),
        TypeKind::float64 => Ok(deserializer.deserialize_float64()?.into_py(py)),
        TypeKind::float128 => Err(XTypesError::InvalidData),
    }
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

fn is_list(member_type: &Bound<PyAny>) -> PyResult<bool> {
    let typing_module = PyModule::import_bound(member_type.py(), "typing")?;
    let origin = typing_module.getattr("get_origin")?.call1((member_type,))?;
    Ok(typing_module.py().get_type_bound::<PyList>().is(&origin))
}

pub struct PythonDdsData {
    pub data: Vec<u8>,
    pub key: Vec<u8>,
}

impl PythonDdsData {
    pub fn from_py_object(py_object: Py<PyAny>) -> PyResult<Self> {
        fn serialize_data_member(
            member_data: &Bound<PyAny>,
            member_type: &Bound<PyAny>,
            serializer: &mut Xcdr1LeSerializer<'_, Vec<u8>>,
        ) -> PyResult<()> {
            if let Ok(member_type_kind) = member_type.extract::<TypeKind>() {
                match member_type_kind {
                    TypeKind::boolean => serializer.serialize_boolean(member_data.extract()?),
                    TypeKind::byte => serializer.serialize_uint8(member_data.extract()?),
                    TypeKind::char8 => serializer.serialize_char8(member_data.extract()?),
                    TypeKind::char16 => serializer.serialize_char8(member_data.extract()?),
                    TypeKind::int8 => serializer.serialize_int8(member_data.extract()?),
                    TypeKind::uint8 => serializer.serialize_uint8(member_data.extract()?),
                    TypeKind::int16 => serializer.serialize_int16(member_data.extract()?),
                    TypeKind::uint16 => serializer.serialize_uint16(member_data.extract()?),
                    TypeKind::int32 => serializer.serialize_int32(member_data.extract()?),
                    TypeKind::uint32 => serializer.serialize_uint32(member_data.extract()?),
                    TypeKind::int64 => serializer.serialize_int64(member_data.extract()?),
                    TypeKind::uint64 => serializer.serialize_uint64(member_data.extract()?),
                    TypeKind::float32 => serializer.serialize_float32(member_data.extract()?),
                    TypeKind::float64 => serializer.serialize_float64(member_data.extract()?),
                    TypeKind::float128 => Err(XTypesError::InvalidData),
                }
                .map_err(|e| PyTypeError::new_err(format!("XTypes error: {:?}", e)))
            } else if is_list(member_type)? {
                let typing_module = PyModule::import_bound(member_type.py(), "typing")?;
                let get_args_attr = typing_module.getattr("get_args")?;
                let type_args = get_args_attr.call1((member_type,))?;
                let type_args = type_args.downcast::<PyTuple>()?;
                let member_type = type_args.get_item(0)?;
                let sequence: &Bound<PySequence> = member_data.downcast::<PySequence>()?;
                let sequence_len = sequence.len()?;
                if type_args.len() != 1 {
                    return Err(PyTypeError::new_err(
                        "Expected list generic with arguments [Type]",
                    ));
                }
                serializer
                    .serialize_uint32(sequence_len as u32)
                    .map_err(|e| PyTypeError::new_err(format!("XTypes error: {:?}", e)))?;
                for index in 0..sequence_len {
                    serialize_data_member(&sequence.get_item(index)?, &member_type, serializer)?;
                }

                Ok(())
            } else if let Ok(py_type) = member_type.downcast::<PyType>() {
                if py_type.py().get_type_bound::<PyBytes>().is(py_type) {
                    serializer
                        .serialize_byte_sequence(member_data.extract()?)
                        .map_err(|e| PyTypeError::new_err(format!("XTypes error: {:?}", e)))
                } else if py_type.py().get_type_bound::<PyString>().is(py_type) {
                    serializer
                        .serialize_string(member_data.extract()?)
                        .map_err(|e| PyTypeError::new_err(format!("XTypes error: {:?}", e)))
                } else {
                    serialize_data(member_type.py(), member_data.clone().unbind(), serializer)
                }
            } else {
                Err(PyTypeError::new_err(format!(
                    "Unsupported Dust DDS representation for Python Type {}",
                    member_type
                )))
            }
        }

        fn serialize_data(
            py: Python<'_>,
            data: Py<PyAny>,
            serializer: &mut Xcdr1LeSerializer<'_, Vec<u8>>,
        ) -> PyResult<()> {
            let annotations = data
                .getattr(py, "__class__")
                .and_then(|c| c.getattr(py, "__annotations__"))?;
            let annotation_dict = annotations
                .downcast_bound::<PyDict>(py)
                .map_err(PyErr::from)?;
            for (member_name, member_type) in annotation_dict {
                let attribute = data.getattr(py, member_name.downcast::<PyString>()?)?;
                serialize_data_member(attribute.bind(py), &member_type, serializer)?;
            }
            Ok(())
        }

        let mut buffer = Vec::new();
        buffer.extend(&CDR_LE);
        buffer.extend(&REPRESENTATION_OPTIONS);
        let mut serializer = Xcdr1LeSerializer::new(&mut buffer);
        Python::with_gil(|py| serialize_data(py, py_object, &mut serializer))?;

        Ok(PythonDdsData {
            data: buffer,
            key: Vec::new(),
        })
    }

    pub fn into_py_object(self, type_: &Py<PyAny>) -> PyResult<Py<PyAny>> {
        fn deserialize_data_member<'de, D>(
            member_type: &Bound<PyAny>,
            deserializer: &mut D,
        ) -> PyResult<Py<PyAny>>
        where
            for<'a> &'a mut D: XTypesDeserializer<'de>,
        {
            let py = member_type.py();
            if let Ok(member_type_kind) = member_type.extract::<TypeKind>() {
                deserialize_into_py_object(py, member_type_kind, deserializer)
                    .map_err(|e| PyTypeError::new_err(format!("XTypesError {:?}", e)))
            } else if is_list(member_type)? {
                let typing_module = PyModule::import_bound(member_type.py(), "typing")?;
                let get_args_attr = typing_module.getattr("get_args")?;
                let type_args = get_args_attr.call1((member_type,))?;
                let type_args = type_args.downcast::<PyTuple>()?;
                let sequence_type = type_args.get_item(0)?;
                let sequence_len = deserializer
                    .deserialize_uint32()
                    .map_err(|e| PyTypeError::new_err(format!("XTypesError {:?}", e)))?;
                let mut list: Vec<Py<PyAny>> = Vec::with_capacity(sequence_len as usize);
                for _ in 0..sequence_len {
                    list.push(deserialize_data_member(&sequence_type, deserializer)?);
                }
                Ok(PyList::new_bound(py, list).into_py(py))
            } else if let Ok(py_type) = member_type.downcast::<PyType>() {
                if py_type.py().get_type_bound::<PyBytes>().is(py_type) {
                    Ok(deserializer
                        .deserialize_byte_sequence()
                        .map_err(|e| PyTypeError::new_err(format!("XTypesError {:?}", e)))?
                        .into_py(py))
                } else if py_type.py().get_type_bound::<PyString>().is(py_type) {
                    Ok(deserializer
                        .deserialize_string()
                        .map_err(|e| PyTypeError::new_err(format!("XTypesError {:?}", e)))?
                        .into_py(py))
                } else {
                    deserialize_data(py, member_type.extract()?, &mut *deserializer)
                }
            } else {
                Err(PyTypeError::new_err(format!(
                    "Unsupported Dust DDS type representation {}",
                    member_type
                )))
            }
        }
        fn deserialize_data<'de, D>(
            py: Python<'_>,
            type_: Py<PyType>,
            deserializer: &mut D,
        ) -> PyResult<Py<PyAny>>
        where
            for<'a> &'a mut D: XTypesDeserializer<'de>,
        {
            let py_type = type_.bind(py);
            let object = type_
                .bind(py)
                .call_method("__new__", (py_type,), None)?
                .unbind();
            let annotations = py_type.getattr("__annotations__")?;
            let annotation_dict = annotations.downcast::<PyDict>().map_err(PyErr::from)?;
            for (member_name, member_type) in annotation_dict {
                let member_name_str = member_name.downcast::<PyString>()?;
                object.setattr(
                    py,
                    member_name_str,
                    deserialize_data_member(&member_type, &mut *deserializer)?,
                )?;
            }
            Ok(object)
        }

        let (header, body) = self.data.split_at(4);
        match [header[0], header[1]] {
            endianness::CDR_LE => Python::with_gil(|py| {
                let type_ = type_.extract(py)?;
                deserialize_data(py, type_, &mut Xcdr1LeDeserializer::new(body))
            }),
            endianness::CDR_BE => Python::with_gil(|py| {
                let type_ = type_.extract(py)?;
                deserialize_data(py, type_, &mut Xcdr1BeDeserializer::new(body))
            }),
            _ => panic!("Unknown endianness"),
        }
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
        None
    }
}
