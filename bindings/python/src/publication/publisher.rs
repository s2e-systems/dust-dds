use pyo3::prelude::*;

#[pyclass]
pub struct Publisher(dust_dds::publication::publisher::Publisher);

impl From<dust_dds::publication::publisher::Publisher> for Publisher {
    fn from(value: dust_dds::publication::publisher::Publisher) -> Self {
        Self(value)
    }
}

impl AsRef<dust_dds::publication::publisher::Publisher> for Publisher {
    fn as_ref(&self) -> &dust_dds::publication::publisher::Publisher {
        &self.0
    }
}
