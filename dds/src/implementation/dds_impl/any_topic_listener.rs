use crate::{
    implementation::utils::shared_object::DdsShared,
    infrastructure::status::InconsistentTopicStatus,
    topic_definition::{topic::Topic, topic_listener::TopicListener},
};

use super::topic_impl::TopicImpl;

pub trait AnyTopicListener {
    fn trigger_on_inconsistent_topic(
        &mut self,
        _the_topic: &DdsShared<TopicImpl>,
        _status: InconsistentTopicStatus,
    );
}

impl<Foo> AnyTopicListener for Box<dyn TopicListener<Foo = Foo> + Send + Sync> {
    fn trigger_on_inconsistent_topic(
        &mut self,
        the_topic: &DdsShared<TopicImpl>,
        status: InconsistentTopicStatus,
    ) {
        self.on_inconsistent_topic(&Topic::new(the_topic.downgrade()), status)
    }
}
