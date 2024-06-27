use dust_dds::{
    infrastructure::{error::DdsResult, instance::InstanceHandle},
    topic_definition::type_support::{DdsDeserialize, DdsHasKey, DdsKey, DdsSerialize, DdsTypeXml},
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
}

#[pyclass]
pub struct PythonDdsData {
    pub data: Vec<u8>,
    pub key: Vec<u8>,
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
