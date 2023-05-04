use crate::{infrastructure::status::InconsistentTopicStatus, topic_definition::topic::Topic};

pub trait TopicListener {
    type Foo;

    fn on_inconsistent_topic(
        &mut self,
        _the_topic: &Topic<Self::Foo>,
        _status: InconsistentTopicStatus,
    ) {
    }
}
