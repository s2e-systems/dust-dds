use dust_dds::infrastructure::type_support::TypeSupport;
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyBytes, PyDict, PyType},
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

pub fn convert_python_type_to_dynamic_type(
    python_type: &Bound<'_, PyAny>,
) -> PyResult<dust_dds::xtypes::dynamic_type::DynamicType> {
    let dataclass_fields = python_type.getattr("__annotations__")?;
    let fields_dict = dataclass_fields.cast::<PyDict>()?;
    let name = python_type.getattr("__name__")?.extract::<String>()?;
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
        let name = field_name.extract()?;
        let r#type = if let Ok(dustdds_type) = field_dict.extract::<TypeKind>() {
            let xtypes_type_kind = match dustdds_type {
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
            dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::get_primitive_type(
                xtypes_type_kind,
            )
        } else {
            let type_name = field_dict.getattr("__name__")?.extract::<String>()?;
            match type_name.as_str() {
                    "int" => dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::get_primitive_type(dust_dds::xtypes::dynamic_type::TypeKind::INT32),
                    "bytes" => dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_sequence_type(
                        dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::get_primitive_type(dust_dds::xtypes::dynamic_type::TypeKind::UINT8),
                        u32::MAX).build(),
                    "str" => dust_dds::xtypes::dynamic_type::DynamicTypeBuilderFactory::create_string_type(u32::MAX).build(),
                   _ => unimplemented!("Mapping not implemented for {type_name}")
                }
        };

        builder
            .add_member(dust_dds::xtypes::dynamic_type::MemberDescriptor {
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
            })
            .map_err(|_| PyRuntimeError::new_err("Failed to add member to dynamic type"))?;
    }

    Ok(builder.build())
}

pub fn convert_python_instance_to_dynamic_data(
    python_instance: Bound<'_, PyAny>,
) -> PyResult<dust_dds::xtypes::dynamic_type::DynamicData> {
    let r#type = convert_python_type_to_dynamic_type(&python_instance.getattr("__class__")?)?;
    let mut dynamic_data = dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(r#type);

    for member_index in 0..dynamic_data.type_ref().get_member_count() {
        let member = dynamic_data
            .type_ref()
            .get_member_by_index(member_index)
            .unwrap();
        let member_descriptor = member.get_descriptor().unwrap();
        let member_kind = member_descriptor.r#type.get_kind();
        let value = python_instance.getattr(member.get_name())?;
        match member_kind {
            dust_dds::xtypes::dynamic_type::TypeKind::NONE => (),
            dust_dds::xtypes::dynamic_type::TypeKind::BOOLEAN => dynamic_data
                .set_boolean_value(member.get_id(), value.extract()?)
                .unwrap(),
            dust_dds::xtypes::dynamic_type::TypeKind::BYTE => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::INT16 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::INT32 => dynamic_data
                .set_int32_value(member.get_id(), value.extract()?)
                .unwrap(),
            dust_dds::xtypes::dynamic_type::TypeKind::INT64 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::UINT16 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::UINT32 => dynamic_data
                .set_uint32_value(member.get_id(), value.extract()?)
                .unwrap(),
            dust_dds::xtypes::dynamic_type::TypeKind::UINT64 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT32 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT64 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT128 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::INT8 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::UINT8 => dynamic_data
                .set_uint8_value(member.get_id(), value.extract()?)
                .unwrap(),
            dust_dds::xtypes::dynamic_type::TypeKind::CHAR8 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::CHAR16 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::STRING8 => dynamic_data
                .set_string_value(member.get_id(), value.extract()?)
                .unwrap(),
            dust_dds::xtypes::dynamic_type::TypeKind::STRING16 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::ALIAS => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::ENUM => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::BITMASK => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::ANNOTATION => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::UNION => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::BITSET => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::SEQUENCE => {
                match member_descriptor
                    .r#type
                    .get_descriptor()
                    .element_type
                    .as_ref()
                    .expect("Should have an element type")
                    .get_kind()
                {
                    dust_dds::xtypes::dynamic_type::TypeKind::UINT8 => dynamic_data
                        .set_uint8_values(member.get_id(), value.extract()?)
                        .unwrap(),
                    kind => todo!("Not implemented for {kind:?}"),
                }
            }
            dust_dds::xtypes::dynamic_type::TypeKind::ARRAY => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::MAP => todo!(),
        }
    }

    Ok(dynamic_data)
}

pub fn convert_dynamic_data_to_python_instance(
    py: Python,
    r#type: &Py<PyAny>,
    dynamic_data: dust_dds::xtypes::dynamic_type::DynamicData,
) -> PyResult<Py<PyAny>> {
    // Call the empty constructor of the type
    let py_type = r#type.cast_bound::<PyType>(py)?;
    let data = r#type.bind(py).call_method("__new__", (py_type,), None)?;

    for member_index in 0..dynamic_data.type_ref().get_member_count() {
        let member = dynamic_data
            .type_ref()
            .get_member_by_index(member_index)
            .expect("Must exist");
        let name = member.get_name();
        let member_descriptor = member.get_descriptor().unwrap();
        match member_descriptor.r#type.get_kind() {
            dust_dds::xtypes::dynamic_type::TypeKind::NONE => (),
            dust_dds::xtypes::dynamic_type::TypeKind::BOOLEAN => {
                data.setattr(
                    name,
                    dynamic_data.get_boolean_value(member.get_id()).unwrap(),
                )?;
            }
            dust_dds::xtypes::dynamic_type::TypeKind::BYTE => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::INT16 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::INT32 => {
                data.setattr(name, dynamic_data.get_int32_value(member.get_id()).unwrap())?;
            }
            dust_dds::xtypes::dynamic_type::TypeKind::INT64 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::UINT16 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::UINT32 => {
                data.setattr(
                    name,
                    dynamic_data.get_uint32_value(member.get_id()).unwrap(),
                )?;
            }
            dust_dds::xtypes::dynamic_type::TypeKind::UINT64 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT32 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT64 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::FLOAT128 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::INT8 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::UINT8 => {
                data.setattr(name, dynamic_data.get_uint8_value(member.get_id()).unwrap())?;
            }
            dust_dds::xtypes::dynamic_type::TypeKind::CHAR8 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::CHAR16 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::STRING8 => {
                data.setattr(
                    name,
                    dynamic_data.get_string_value(member.get_id()).unwrap(),
                )?;
            }
            dust_dds::xtypes::dynamic_type::TypeKind::STRING16 => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::ALIAS => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::ENUM => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::BITMASK => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::ANNOTATION => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::UNION => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::BITSET => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::SEQUENCE => {
                match member_descriptor
                    .r#type
                    .get_descriptor()
                    .element_type
                    .as_ref()
                    .expect("Should have an element type")
                    .get_kind()
                {
                    dust_dds::xtypes::dynamic_type::TypeKind::UINT8 => {
                        data.setattr(
                            name,
                            PyBytes::new(
                                py,
                                dynamic_data
                                    .get_uint8_values(member.get_id())
                                    .unwrap()
                                    .as_slice(),
                            ),
                        )?;
                    }
                    kind => todo!("Not implemented for {kind:?}"),
                }
            }
            dust_dds::xtypes::dynamic_type::TypeKind::ARRAY => todo!(),
            dust_dds::xtypes::dynamic_type::TypeKind::MAP => todo!(),
        }
    }

    Ok(data.unbind())
}

pub struct PythonDdsData(dust_dds::xtypes::dynamic_type::DynamicData);

impl From<dust_dds::xtypes::dynamic_type::DynamicData> for PythonDdsData {
    fn from(value: dust_dds::xtypes::dynamic_type::DynamicData) -> Self {
        Self(value)
    }
}

impl From<PythonDdsData> for dust_dds::xtypes::dynamic_type::DynamicData {
    fn from(value: PythonDdsData) -> Self {
        value.0
    }
}

impl TypeSupport for PythonDdsData {
    fn get_type() -> dust_dds::xtypes::dynamic_type::DynamicType {
        todo!()
    }

    fn create_sample(src: dust_dds::xtypes::dynamic_type::DynamicData) -> Self {
        Self(src)
    }

    fn create_dynamic_sample(self) -> dust_dds::xtypes::dynamic_type::DynamicData {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use dust_dds::infrastructure::type_support::DdsType;
    use pyo3::ffi::c_str;

    use super::*;

    #[test]
    fn test_convert_python_to_dynamic_type() {
        Python::initialize();
        Python::attach(|py| {
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
            let dynamic_type = convert_python_type_to_dynamic_type(&my_data_type).unwrap();

            assert_eq!(dynamic_type.get_name(), "MyDataType");
            assert_eq!(dynamic_type.get_member_count(), 4);

            let member = dynamic_type.get_member_by_index(0).unwrap();
            assert_eq!(member.get_name(), "id");
            assert_eq!(
                member.get_descriptor().unwrap().r#type.get_kind(),
                dust_dds::xtypes::dynamic_type::TypeKind::UINT8
            );

            let member = dynamic_type.get_member_by_index(1).unwrap();
            assert_eq!(member.get_name(), "value");
            assert_eq!(
                member.get_descriptor().unwrap().r#type.get_kind(),
                dust_dds::xtypes::dynamic_type::TypeKind::INT32
            );

            let member = dynamic_type.get_member_by_index(2).unwrap();
            assert_eq!(member.get_name(), "data");
            assert_eq!(
                member.get_descriptor().unwrap().r#type.get_kind(),
                dust_dds::xtypes::dynamic_type::TypeKind::SEQUENCE
            );

            let member = dynamic_type.get_member_by_index(3).unwrap();
            assert_eq!(member.get_name(), "name");
            assert_eq!(
                member.get_descriptor().unwrap().r#type.get_kind(),
                dust_dds::xtypes::dynamic_type::TypeKind::STRING8
            );
        });
    }

    #[test]
    fn test_convert_python_instance_to_dynamic_data() {
        Python::initialize();
        Python::attach(|py| {
            let code = c_str!(
                r"
from dataclasses import dataclass

@dataclass
class MyDataType:
    id: TypeKind.uint8
    value: int
    data: bytes
    name: str

my_data = MyDataType(id=10, value=100, data=bytes([0,1,2,3,4]), name='mytype')
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
            let my_data_type = locals.get_item("my_data").unwrap().unwrap();
            let dynamic_data = convert_python_instance_to_dynamic_data(my_data_type).unwrap();

            assert_eq!(dynamic_data.get_uint8_value(0).unwrap(), &10);
            assert_eq!(dynamic_data.get_int32_value(1).unwrap(), &100);
            assert_eq!(
                dynamic_data.get_uint8_values(2).unwrap(),
                vec![0, 1, 2, 3, 4]
            );
            assert_eq!(
                dynamic_data.get_string_value(3).unwrap(),
                &String::from("mytype")
            );
        });
    }

    #[test]
    fn test_convert_dynamic_data_to_python() {
        #[derive(DdsType)]
        pub struct MyDataType {
            id: u8,
            value: i32,
            data: Vec<u8>,
            name: String,
        }

        Python::initialize();
        Python::attach(|py| {
            let code = c_str!(
                r"
from dataclasses import dataclass

@dataclass
class MyDataType:
    id: TypeKind.uint8
    value: int
    data: bytes
    name: str
    "
            );

            let dynamic_data = MyDataType {
                id: 10,
                value: 100,
                data: vec![1, 2, 3, 4],
                name: String::from("Myname"),
            }
            .create_dynamic_sample();

            let globals = PyDict::new(py);
            globals
                .set_item(
                    "TypeKind",
                    py.get_type::<TypeKind>().into_pyobject(py).unwrap(),
                )
                .unwrap();
            let locals = PyDict::new(py);
            py.run(code, Some(&globals), Some(&locals)).unwrap();
            let r#type = locals.get_item("MyDataType").unwrap().unwrap().unbind();
            let created_data =
                convert_dynamic_data_to_python_instance(py, &r#type, dynamic_data).unwrap();

            assert_eq!(
                created_data
                    .getattr(py, "id")
                    .unwrap()
                    .extract::<u8>(py)
                    .unwrap(),
                10
            );
            assert_eq!(
                created_data
                    .getattr(py, "value")
                    .unwrap()
                    .extract::<i32>(py)
                    .unwrap(),
                100
            );

            assert_eq!(
                created_data
                    .getattr(py, "data")
                    .unwrap()
                    .extract::<Vec<u8>>(py)
                    .unwrap(),
                vec![1, 2, 3, 4]
            );

            assert_eq!(
                created_data
                    .getattr(py, "name")
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap(),
                "Myname"
            );

            let code = c_str!(
                r"
assert created_data.id==10
assert created_data.value==100
assert created_data.data==bytes([1,2,3,4])
assert created_data.name=='Myname'
    "
            );
            let locals = PyDict::new(py);
            locals.set_item("created_data", created_data).unwrap();
            let globals = PyDict::new(py);
            globals
                .set_item(
                    "TypeKind",
                    py.get_type::<TypeKind>().into_pyobject(py).unwrap(),
                )
                .unwrap();
            py.run(&code, Some(&globals), Some(&locals)).unwrap();
        });
    }
}
