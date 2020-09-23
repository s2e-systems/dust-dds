use std::any::Any;
use crate::infrastructure::listener::NoListener;
pub trait PublisherListener: Any + Send + Sync{}

impl PublisherListener for NoListener{}