use crate::{
    infrastructure::status::InconsistentTopicStatus,
    topic_definition::{topic::Topic, topic_listener::TopicListener},
};

use super::nodes::{TopicNode, TopicNodeKind};

pub trait AnyTopicListener {
    fn trigger_on_inconsistent_topic(
        &mut self,
        _the_topic: TopicNode,
        _status: InconsistentTopicStatus,
    );
}

impl<Foo> AnyTopicListener for Box<dyn TopicListener<Foo = Foo> + Send + Sync> {
    fn trigger_on_inconsistent_topic(
        &mut self,
        the_topic: TopicNode,
        status: InconsistentTopicStatus,
    ) {
        self.on_inconsistent_topic(&Topic::new(TopicNodeKind::Listener(the_topic)), status)
    }
}
