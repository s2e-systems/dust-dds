use pyo3::prelude::*;

#[pyclass]
pub struct Subscriber(dust_dds::subscription::subscriber::Subscriber);

impl From<dust_dds::subscription::subscriber::Subscriber> for Subscriber {
    fn from(value: dust_dds::subscription::subscriber::Subscriber) -> Self {
        Self(value)
    }
}

impl AsRef<dust_dds::subscription::subscriber::Subscriber> for Subscriber {
    fn as_ref(&self) -> &dust_dds::subscription::subscriber::Subscriber {
        &self.0
    }
}
