use dust_dds::infrastructure::error::DdsError;
use pyo3::{PyErr, exceptions::PyTypeError};

pub fn into_pyerr(e: DdsError) -> PyErr {
    PyTypeError::new_err(format!("{e:?}"))
}
