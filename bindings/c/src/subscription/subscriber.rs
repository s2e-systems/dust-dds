/// cbindgen:opaque
pub struct DustDdsSubscriber(pub(crate) dust_dds::subscription::subscriber::Subscriber);

pub type Subscriber = DustDdsSubscriber;

impl DustDdsSubscriber {
    pub fn new(subscriber: dust_dds::subscription::subscriber::Subscriber) -> Self {
        Self(subscriber)
    }

    pub fn inner(&self) -> &dust_dds::subscription::subscriber::Subscriber {
        &self.0
    }
}
