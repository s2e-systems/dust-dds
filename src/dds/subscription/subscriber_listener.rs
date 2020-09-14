use std::any::Any;
use crate::dds::subscription::subscriber::Subscriber;

pub trait SubscriberListener: Any + Send + Sync {
    fn on_data_on_readers(&self, _the_subscriber: Subscriber);
}