use std::any::Any;

pub trait PublisherListener: Any + Send + Sync{}