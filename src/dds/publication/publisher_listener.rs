use std::any::Any;
use crate::dds::infrastructure::listener::NoListener;
pub trait PublisherListener: Any + Send + Sync{}

impl PublisherListener for NoListener{}