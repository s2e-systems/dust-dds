use dust_dds::infrastructure::type_support::TypeSupport;
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyBytes, PyDict, PyList, PyString, PyType},
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

#[allow(dead_code)]
pub struct PythonTypeRepresentation(dust_dds::xtypes::type_object::CompleteTypeObject);
impl From<PythonTypeRepresentation> for dust_dds::xtypes::dynamic_type::DynamicType {
    fn from(_value: PythonTypeRepresentation) -> Self {
        todo!()
    }
}

impl TryFrom<Py<PyAny>> for PythonTypeRepresentation {
    type Error = PyErr;

    fn try_from(value: Py<PyAny>) -> PyResult<Self> {
        let type_name = Python::attach(|py| {
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
                .cast_bound::<PyDict>(py)?
                .values()
                .len())
        }

        let member_count = Python::attach(|py| get_member_count(py, &value))
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
                    .cast_bound::<PyDict>(py)?
                    .values();
                let field = dataclass_fields.get_item(index)?;

                Ok(field.getattr("name")?.to_string())
            }
            let name = Python::attach(|py| get_member_name(py, &value, index))?;

            let is_key = false; //TODO!

            fn get_member_type(
                py: Python<'_>,
                type_representation: &Py<PyAny>,
                index: usize,
            ) -> PyResult<dust_dds::xtypes::type_object::TypeIdentifier> {
                let dataclass_fields = type_representation
                    .getattr(py, "__dataclass_fields__")?
                    .cast_bound::<PyDict>(py)?
                    .values();
                let field = dataclass_fields.get_item(index)?;

                let type_value = field.getattr("type")?;

                if let Ok(type_kind) = type_value.extract::<TypeKind>() {
                    Ok(type_kind.into())
                } else if is_list(&type_value)? {
                    todo!()
                } else if let Ok(py_type) = type_value.cast::<PyType>() {
                    if py_type.py().get_type::<PyBytes>().is(py_type) {
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
                    } else if py_type.py().get_type::<PyString>().is(py_type) {
                        Ok(
                            dust_dds::xtypes::type_object::TypeIdentifier::TiString8Small {
                                string_sdefn: dust_dds::xtypes::type_object::StringSTypeDefn {
                                    bound: 0,
                                },
                            },
                        )
                    } else {
                        Err(PyTypeError::new_err(format!(
                            "Unsupported Dust DDS representation for Python Type {py_type}"
                        )))
                    }
                } else {
                    Err(PyTypeError::new_err(format!(
                        "Unsupported Dust DDS representation for Python Type {type_value}"
                    )))
                }
            }
            let member_type_id = Python::attach(|py| get_member_type(py, &value, index))?;

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
    let typing_module = PyModule::import(member_type.py(), "typing")?;
    let origin = typing_module.getattr("get_origin")?.call1((member_type,))?;
    Ok(typing_module.py().get_type::<PyList>().is(&origin))
}

pub struct PythonDdsData {
    pub data: Vec<u8>,
    pub key: Vec<u8>,
}
impl TypeSupport for PythonDdsData {
    fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
        todo!()
    }

    fn create_sample(_src: dust_dds::xtypes::dynamic_type::DynamicData) -> Self {
        todo!()
    }

    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        todo!()
    }
}

impl PythonDdsData {
    pub fn from_py_object(_py_object: Py<PyAny>) -> PyResult<Self> {
        // fn serialize_data_member(
        //     _member_data: &Bound<PyAny>,
        //     _member_type: &Bound<PyAny>,
        //     _serializer: &mut Xcdr1LeSerializer<Vec<u8>>,
        // ) -> PyResult<()> {
        //     todo!()
        // }

        // fn serialize_data(
        //     py: Python<'_>,
        //     data: Py<PyAny>,
        //     serializer: &mut Xcdr1LeSerializer<Vec<u8>>,
        // ) -> PyResult<()> {
        //     let annotations = data
        //         .getattr(py, "__class__")
        //         .and_then(|c| c.getattr(py, "__annotations__"))?;
        //     let annotation_dict = annotations
        //         .cast_bound_bound::<PyDict>(py)
        //         .map_err(PyErr::from)?;
        //     for (member_name, member_type) in annotation_dict {
        //         let attribute = data.getattr(py, member_name.cast_bound::<PyString>()?)?;
        //         serialize_data_member(attribute.bind(py), &member_type, serializer)?;
        //     }
        //     Ok(())
        // }

        // let mut buffer = Vec::new();
        // buffer.extend(&CDR_LE);
        // buffer.extend(&REPRESENTATION_OPTIONS);
        // let mut serializer = Xcdr1LeSerializer::new(buffer);
        // Python::attach(|py| serialize_data(py, py_object, &mut serializer))?;

        // Ok(PythonDdsData {
        //     data: serializer.into_inner(),
        //     key: Vec::new(),
        // })
        todo!()
    }

    pub fn into_py_object(self, _type_: &Py<PyAny>) -> PyResult<Py<PyAny>> {

        // let (header, body) = self.data.split_at(4);
        // match [header[0], header[1]] {
        //     endianness::CDR_LE => Python::attach(|py| {
        //         let type_ = type_.extract(py)?;
        //         deserialize_data(py, type_, &mut Xcdr1LeDeserializer::new(body))
        //     }),
        //     endianness::CDR_BE => Python::attach(|py| {
        //         let type_ = type_.extract(py)?;
        //         deserialize_data(py, type_, &mut Xcdr1BeDeserializer::new(body))
        //     }),
        //     _ => panic!("Unknown endianness"),
        // }
        todo!()
    }
}
