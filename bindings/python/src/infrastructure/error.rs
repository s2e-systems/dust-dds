use dust_dds::infrastructure::error::DdsError;
use pyo3::{exceptions::PyTypeError, PyErr};

pub fn into_pyerr(e: DdsError) -> PyErr {
    PyTypeError::new_err(format!("{:?}", e))
}
