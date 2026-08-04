/// cbindgen:opaque
pub struct ContentFilteredTopic(
    pub(crate) dust_dds::topic_definition::content_filtered_topic::ContentFilteredTopic,
);

impl ContentFilteredTopic {
    pub fn new(
        content_filtered_topic: dust_dds::topic_definition::content_filtered_topic::ContentFilteredTopic,
    ) -> Self {
        Self(content_filtered_topic)
    }

    pub fn inner(
        &self,
    ) -> &dust_dds::topic_definition::content_filtered_topic::ContentFilteredTopic {
        &self.0
    }
}
