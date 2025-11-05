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
    char8,
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
}

impl From<TypeKind> for dust_dds::xtypes::type_object::TypeIdentifier {
    fn from(value: TypeKind) -> Self {
        match value {
            TypeKind::boolean => dust_dds::xtypes::type_object::TypeIdentifier::TkBoolean,
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
            TypeKind::char8 => dust_dds::xtypes::type_object::TypeIdentifier::TkChar8Type,
        }
    }
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

#[cfg(test)]
mod tests {
    use pyo3::ffi::c_str;

    use super::*;

    fn convert_python_to_dynamic_type(
        python_type: Bound<'_, PyAny>,
    ) -> dust_dds::xtypes::error::XTypesResult<dust_dds::xtypes::dynamic_type::DynamicType> {
        let dataclass_fields = python_type
            .getattr("__annotations__")
            .map_err(|_| dust_dds::xtypes::error::XTypesError::InvalidType)?;
        let fields_dict = dataclass_fields
            .cast::<PyDict>()
            .map_err(|_| dust_dds::xtypes::error::XTypesError::InvalidType)?;
        let name = python_type
            .getattr("__name__")
            .map_err(|_| dust_dds::xtypes::error::XTypesError::InvalidType)?
            .extract::<String>()
            .map_err(|_| dust_dds::xtypes::error::XTypesError::InvalidType)?;
        let mut builder = dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_type(
            dust_dds::xtypes::dynamic_type::TypeDescriptor {
                kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
                name,
                base_type: None,
                discriminator_type: None,
                bound: Vec::new(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
                is_nested: false,
            },
        );

        for (index, (field_name, field_dict)) in fields_dict.iter().enumerate() {
            let name = field_name
                .extract()
                .map_err(|_| dust_dds::xtypes::error::XTypesError::InvalidType)?;
            let xtypes_type_kind = match field_dict
                .extract::<TypeKind>()
                .map_err(|_| dust_dds::xtypes::error::XTypesError::InvalidType)?
            {
                TypeKind::boolean => dust_dds::xtypes::dynamic_type::TypeKind::BOOLEAN,
                TypeKind::char8 => dust_dds::xtypes::dynamic_type::TypeKind::CHAR8,
                TypeKind::int8 => dust_dds::xtypes::dynamic_type::TypeKind::INT8,
                TypeKind::uint8 => dust_dds::xtypes::dynamic_type::TypeKind::UINT8,
                TypeKind::int16 => dust_dds::xtypes::dynamic_type::TypeKind::INT16,
                TypeKind::uint16 => dust_dds::xtypes::dynamic_type::TypeKind::UINT16,
                TypeKind::int32 => dust_dds::xtypes::dynamic_type::TypeKind::INT32,
                TypeKind::uint32 => dust_dds::xtypes::dynamic_type::TypeKind::UINT32,
                TypeKind::int64 => dust_dds::xtypes::dynamic_type::TypeKind::INT64,
                TypeKind::uint64 => dust_dds::xtypes::dynamic_type::TypeKind::UINT64,
                TypeKind::float32 => dust_dds::xtypes::dynamic_type::TypeKind::FLOAT32,
                TypeKind::float64 => dust_dds::xtypes::dynamic_type::TypeKind::FLOAT64,
            };
            let r#type =
                dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::get_primitive_type(
                    xtypes_type_kind,
                );

            builder.add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
                name,
                id: index as u32,
                r#type,
                default_value: None,
                index: index as u32,
                label: vec![],
                try_construct_kind: dust_dds::xtypes::dynamic_type::TryConstructKind::UseDefault,
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })?;
        }

        Ok(builder.build())
    }

    #[test]
    fn test_convert_python_to_dynamic_type() {
        Python::initialize();
        let python_type = Python::attach(|py| {
            let code = c_str!(
                r"
class MyDataType:
    id: TypeKind.uint8
    value: int
    data: bytes
    name: str
    "
            );
            let locals = PyDict::new(py);
            let globals = PyDict::new(py);
            globals
                .set_item(
                    "TypeKind",
                    py.get_type::<TypeKind>().into_pyobject(py).unwrap(),
                )
                .unwrap();
            py.run(code, Some(&globals), Some(&locals)).unwrap();
            let my_data_type = locals.get_item("MyDataType").unwrap().unwrap();
            let dynamic_type = convert_python_to_dynamic_type(my_data_type).unwrap();

            assert_eq!(dynamic_type.get_name(), "MyDataType");
            assert_eq!(dynamic_type.get_member_count(), 4);

            let id_member = dynamic_type.get_member_by_index(0).unwrap();
            assert_eq!(id_member.get_name(), "id");
            assert_eq!(
                id_member.get_descriptor().unwrap().r#type.get_kind(),
                dust_dds::xtypes::dynamic_type::TypeKind::UINT8
            );

            let data_member = dynamic_type.get_member_by_index(1).unwrap();
            assert_eq!(data_member.get_name(), "value");
            assert_eq!(
                id_member.get_descriptor().unwrap().r#type.get_kind(),
                dust_dds::xtypes::dynamic_type::TypeKind::INT64
            );

            let data_member = dynamic_type.get_member_by_index(2).unwrap();
            assert_eq!(data_member.get_name(), "data");
            assert_eq!(
                id_member.get_descriptor().unwrap().r#type.get_kind(),
                dust_dds::xtypes::dynamic_type::TypeKind::SEQUENCE
            );

            let data_member = dynamic_type.get_member_by_index(3).unwrap();
            assert_eq!(data_member.get_name(), "name");
            assert_eq!(
                id_member.get_descriptor().unwrap().r#type.get_kind(),
                dust_dds::xtypes::dynamic_type::TypeKind::STRING8
            );
        });
    }
}
