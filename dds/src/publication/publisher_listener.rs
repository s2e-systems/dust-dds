use crate::infrastructure::listener::NoListener;
pub trait PublisherListener{}

impl PublisherListener for NoListener{}