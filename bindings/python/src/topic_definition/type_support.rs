use dust_dds::topic_definition::type_support::{
    DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml,
};
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

#[pyclass]
pub struct PythonDdsData(Vec<u8>);

impl DdsHasKey for PythonDdsData {
    const HAS_KEY: bool = true;
}

impl DdsKey for PythonDdsData {
    type Key = ();

    fn get_key(&self) -> dust_dds::infrastructure::error::DdsResult<Self::Key> {
        todo!()
    }

    fn get_key_from_serialized_data(
        serialized_foo: &[u8],
    ) -> dust_dds::infrastructure::error::DdsResult<Self::Key> {
        todo!()
    }
}

impl DdsSerialize for PythonDdsData {
    fn serialize_data(&self) -> dust_dds::infrastructure::error::DdsResult<Vec<u8>> {
        todo!()
    }
}

impl<'de> DdsDeserialize<'de> for PythonDdsData {
    fn deserialize_data(
        _serialized_data: &'de [u8],
    ) -> dust_dds::infrastructure::error::DdsResult<Self> {
        todo!()
    }
}

impl DdsTypeXml for PythonDdsData {
    fn get_type_xml() -> Option<String> {
        todo!()
    }
}

#[pymethods]
impl PythonDdsData {
    #[new]
    fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    #[getter]
    fn get_value(&self) -> &[u8] {
        &self.0
    }
}
