use pyo3::prelude::*;

#[allow(non_camel_case_types)]
#[pyclass]
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
        todo!()
    }

    fn get_serialized_key_from_serialized_foo(
        &self,
        _serialized_foo: &[u8],
    ) -> dust_dds::infrastructure::error::DdsResult<Vec<u8>> {
        todo!()
    }

    fn instance_handle_from_serialized_foo(
        &self,
        _serialized_foo: &[u8],
    ) -> dust_dds::infrastructure::error::DdsResult<
        dust_dds::infrastructure::instance::InstanceHandle,
    > {
        todo!()
    }

    fn instance_handle_from_serialized_key(
        &self,
        _serialized_key: &[u8],
    ) -> dust_dds::infrastructure::error::DdsResult<
        dust_dds::infrastructure::instance::InstanceHandle,
    > {
        todo!()
    }

    fn xml_type(&self) -> String {
        todo!()
    }
}

#[derive(dust_dds::topic_definition::type_support::DdsType)]
#[pyclass]
pub struct MyDdsData(Vec<u8>);

#[pymethods]
impl MyDdsData {
    #[new]
    fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    #[getter]
    fn get_value(&self) -> &[u8] {
        &self.0
    }
}
