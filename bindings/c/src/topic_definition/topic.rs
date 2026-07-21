/// cbindgen:opaque
pub struct DustDdsTopic(pub(crate) dust_dds::topic_definition::topic::Topic);

pub type Topic = DustDdsTopic;

impl DustDdsTopic {
    pub fn new(topic: dust_dds::topic_definition::topic::Topic) -> Self {
        Self(topic)
    }

    pub fn inner(&self) -> &dust_dds::topic_definition::topic::Topic {
        &self.0
    }
}
