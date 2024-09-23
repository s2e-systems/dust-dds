use crate::xtypes::endianness::{self, CDR_LE, REPRESENTATION_OPTIONS};
use dust_dds::{
    infrastructure::error::DdsResult,
    topic_definition::type_support::{DdsDeserialize, DdsSerialize},
    xtypes::{
        deserializer::XTypesDeserializer, error::XTypesError, serializer::XTypesSerializer,
        xcdr_deserializer::Xcdr1BeDeserializer, xcdr_deserializer::Xcdr1LeDeserializer,
        xcdr_serializer::Xcdr1LeSerializer,
    },
};
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyBytes, PyDict, PyList, PySequence, PyString, PyTuple, PyType},
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

impl From<TypeKind> for dust_dds::xtypes::type_object::TypeIdentifier {
    fn from(value: TypeKind) -> Self {
        match value {
            TypeKind::boolean => dust_dds::xtypes::type_object::TypeIdentifier::TkBoolean,
            TypeKind::byte => dust_dds::xtypes::type_object::TypeIdentifier::TkByteType,
            TypeKind::int8 => dust_dds::xtypes::type_object::TypeIdentifier::TkInt8Type,
            TypeKind::uint8 => dust_dds::xtypes::type_object::TypeIdentifier::TkInt16Type,
            TypeKind::int16 => dust_dds::xtypes::type_object::TypeIdentifier::TkInt32Type,
            TypeKind::uint16 => dust_dds::xtypes::type_object::TypeIdentifier::TkInt64Type,
            TypeKind::int32 => dust_dds::xtypes::type_object::TypeIdentifier::TkUint8Type,
            TypeKind::uint32 => dust_dds::xtypes::type_object::TypeIdentifier::TkUint16Type,
            TypeKind::int64 => dust_dds::xtypes::type_object::TypeIdentifier::TkUint32Type,
            TypeKind::uint64 => dust_dds::xtypes::type_object::TypeIdentifier::TkUint64Type,
            TypeKind::float32 => dust_dds::xtypes::type_object::TypeIdentifier::TkFloat32Type,
            TypeKind::float64 => dust_dds::xtypes::type_object::TypeIdentifier::TkFloat64Type,
            TypeKind::float128 => dust_dds::xtypes::type_object::TypeIdentifier::TkFloat128Type,
            TypeKind::char8 => dust_dds::xtypes::type_object::TypeIdentifier::TkChar8Type,
            TypeKind::char16 => dust_dds::xtypes::type_object::TypeIdentifier::TkChar16Type,
        }
    }
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

pub struct PythonTypeRepresentation(dust_dds::xtypes::type_object::CompleteTypeObject);
impl dust_dds::xtypes::dynamic_type::DynamicType for PythonTypeRepresentation {
    fn get_descriptor(
        &self,
    ) -> Result<dust_dds::xtypes::dynamic_type::TypeDescriptor, XTypesError> {
        self.0.get_descriptor()
    }

    fn get_name(&self) -> dust_dds::xtypes::dynamic_type::ObjectName {
        self.0.get_name()
    }

    fn get_kind(&self) -> dust_dds::xtypes::type_object::TypeKind {
        self.0.get_kind()
    }

    fn get_member_count(&self) -> u32 {
        self.0.get_member_count()
    }

    fn get_member_by_index(
        &self,
        index: u32,
    ) -> Result<&dyn dust_dds::xtypes::dynamic_type::DynamicTypeMember, XTypesError> {
        self.0.get_member_by_index(index)
    }
}

impl TryFrom<Py<PyAny>> for PythonTypeRepresentation {
    type Error = PyErr;

    fn try_from(value: Py<PyAny>) -> PyResult<Self> {
        let type_name = Python::with_gil(|py| {
            value
                .bind(py)
                .get_type()
                .name()
                .expect("name exists")
                .to_string()
        });

        fn get_member_count(py: Python<'_>, python_type: &Py<PyAny>) -> PyResult<usize> {
            Ok(python_type
                .getattr(py, "__dataclass_fields__")?
                .downcast_bound::<PyDict>(py)?
                .values()
                .len())
        }

        let member_count = Python::with_gil(|py| get_member_count(py, &value))
            .expect("Should be able to get member size");

        let mut member_seq = Vec::new();
        for index in 0..member_count {
            fn get_member_name(
                py: Python<'_>,
                type_representation: &Py<PyAny>,
                index: usize,
            ) -> PyResult<String> {
                let dataclass_fields = type_representation
                    .getattr(py, "__dataclass_fields__")?
                    .downcast_bound::<PyDict>(py)?
                    .values();
                let field = dataclass_fields.get_item(index as usize)?;

                Ok(field.getattr("name")?.to_string())
            }
            let name = Python::with_gil(|py| get_member_name(py, &value, index))?;

            let is_key = false; //TODO!

            fn get_member_type(
                py: Python<'_>,
                type_representation: &Py<PyAny>,
                index: usize,
            ) -> PyResult<dust_dds::xtypes::type_object::TypeIdentifier> {
                let dataclass_fields = type_representation
                    .getattr(py, "__dataclass_fields__")?
                    .downcast_bound::<PyDict>(py)?
                    .values();
                let field = dataclass_fields.get_item(index as usize)?;

                let type_value = field.getattr("type")?;

                if let Ok(type_kind) = type_value.extract::<TypeKind>() {
                    Ok(type_kind.into())
                } else if is_list(&type_value)? {
                    todo!()
                } else {
                    if let Ok(py_type) = type_value.downcast::<PyType>() {
                        if py_type.py().get_type_bound::<PyBytes>().is(py_type) {
                            Ok(dust_dds::xtypes::type_object::TypeIdentifier::TiPlainSequenceSmall {
                                seq_sdefn: Box::new(dust_dds::xtypes::type_object::PlainSequenceSElemDefn {
                                        header: dust_dds::xtypes::type_object::PlainCollectionHeader {
                                            equiv_kind: dust_dds::xtypes::type_object::EK_COMPLETE,
                                            element_flags: dust_dds::xtypes::type_object::CollectionElementFlag {
                                                try_construct: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                                                is_external: false
                                            },
                                        },
                                        bound: 0,
                                        element_identifier: dust_dds::xtypes::type_object::TypeIdentifier::TkUint8Type,
                                    })
                                })
                        } else if py_type.py().get_type_bound::<PyString>().is(py_type) {
                            Ok(
                                dust_dds::xtypes::type_object::TypeIdentifier::TiString8Small {
                                    string_sdefn: dust_dds::xtypes::type_object::StringSTypeDefn {
                                        bound: 0,
                                    },
                                },
                            )
                        } else {
                            Err(PyTypeError::new_err(format!(
                                "Unsupported Dust DDS representation for Python Type {}",
                                py_type
                            )))
                        }
                    } else {
                        Err(PyTypeError::new_err(format!(
                            "Unsupported Dust DDS representation for Python Type {}",
                            type_value
                        )))
                    }
                }
            }
            let member_type_id = Python::with_gil(|py| get_member_type(py, &value, index))?;

            member_seq.push(dust_dds::xtypes::type_object::CompleteStructMember {
                common: dust_dds::xtypes::type_object::CommonStructMember {
                    member_id: index as u32,
                    member_flags: dust_dds::xtypes::type_object::StructMemberFlag {
                        try_construct: dust_dds::xtypes::dynamic_type::TryConstructKind::Discard,
                        is_external: false,
                        is_optional: false,
                        is_must_undestand: true,
                        is_key,
                    },
                    member_type_id,
                },
                detail: dust_dds::xtypes::type_object::CompleteMemberDetail {
                    name,
                    ann_builtin: None,
                    ann_custom: None,
                },
            });
        }

        Ok(Self(
            dust_dds::xtypes::type_object::CompleteTypeObject::TkStructure {
                struct_type: dust_dds::xtypes::type_object::CompleteStructType {
                    struct_flags: dust_dds::xtypes::type_object::StructTypeFlag {
                        is_final: true,
                        is_appendable: false,
                        is_mutable: false,
                        is_nested: false,
                        is_autoid_hash: false,
                    },
                    header: dust_dds::xtypes::type_object::CompleteStructHeader {
                        base_type: dust_dds::xtypes::type_object::TypeIdentifier::TkNone,
                        detail: dust_dds::xtypes::type_object::CompleteTypeDetail {
                            ann_builtin: None,
                            ann_custom: None,
                            type_name,
                        },
                    },
                    member_seq,
                },
            },
        ))
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
