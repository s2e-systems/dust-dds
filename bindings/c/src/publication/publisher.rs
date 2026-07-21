/// cbindgen:opaque
pub struct DustDdsPublisher(pub(crate) dust_dds::publication::publisher::Publisher);

pub type Publisher = DustDdsPublisher;

impl DustDdsPublisher {
    pub fn new(publisher: dust_dds::publication::publisher::Publisher) -> Self {
        Self(publisher)
    }

    pub fn inner(&self) -> &dust_dds::publication::publisher::Publisher {
        &self.0
    }
}
