use crate::dds::subscription::subscriber::Subscriber;

pub trait SubscriberListener {
    fn on_data_on_readers(&self, _the_subscriber: Subscriber);
}