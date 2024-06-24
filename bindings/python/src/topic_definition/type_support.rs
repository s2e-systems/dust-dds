use pyo3::prelude::*;

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
