use dust_dds::serialized_payload::cdr::serializer::CdrSerializer;
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyDict};

use crate::{
    topic_definition::type_support::PythonDdsData,
    xtypes::{
        cdr_serializer::ClassicCdrSerializer,
        endianness::{CdrEndianness, CDR_LE, REPRESENTATION_OPTIONS},
    },
};

#[pyclass]
pub struct DataWriter(dust_dds::publication::data_writer::DataWriter<PythonDdsData>);

impl From<dust_dds::publication::data_writer::DataWriter<PythonDdsData>> for DataWriter {
    fn from(value: dust_dds::publication::data_writer::DataWriter<PythonDdsData>) -> Self {
        Self(value)
    }
}

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
    // for (member_id, member_type) in annotation_dict {
    serializer
        .serialize_u32(data.getattr(py, "id")?.extract::<u32>(py)?)
        .unwrap();
    serializer
        .serialize_bool(data.getattr(py, "state")?.extract::<bool>(py)?)
        .unwrap();
    // data.getattr(py, member_id.to_string());
    // }
    Ok(())
}

#[pymethods]
impl DataWriter {
    pub fn write(&self, data: Py<PyAny>) -> PyResult<()> {
        let mut buffer = Vec::new();
        buffer.extend(&CDR_LE);
        buffer.extend(&REPRESENTATION_OPTIONS);
        let mut serializer = ClassicCdrSerializer::new(&mut buffer, CdrEndianness::LittleEndian);
        let _ = Python::with_gil(|py| serialize_data(py, data, &mut serializer))?;

        let dds_data = PythonDdsData {
            data: buffer,
            key: Vec::new(),
        };
        match self.0.write(&dds_data, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e))),
        }
    }
}
