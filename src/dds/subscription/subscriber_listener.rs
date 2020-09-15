use std::any::Any;
use crate::dds::subscription::subscriber::Subscriber;
use crate::dds::infrastructure::listener::NoListener;

pub trait SubscriberListener: Any + Send + Sync {
    fn on_data_on_readers(&self, _the_subscriber: Subscriber);
}

impl SubscriberListener for NoListener {
    fn on_data_on_readers(&self, _the_subscriber: Subscriber) {
        todo!()
    }
}