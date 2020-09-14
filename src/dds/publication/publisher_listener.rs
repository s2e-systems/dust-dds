use std::any::Any;

pub trait PublisherListener: Any + Send + Sync{}

pub struct NoPublisherListener;

impl PublisherListener for NoPublisherListener{}