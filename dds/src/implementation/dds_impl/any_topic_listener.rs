use crate::{
    infrastructure::status::InconsistentTopicStatus,
    topic_definition::{topic::Topic, topic_listener::TopicListener},
};

use super::{node_kind::TopicNodeKind, node_listener_topic::ListenerTopicNode};

pub trait AnyTopicListener {
    fn trigger_on_inconsistent_topic(
        &mut self,
        _the_topic: ListenerTopicNode,
        _status: InconsistentTopicStatus,
    );
}

impl<Foo> AnyTopicListener for Box<dyn TopicListener<Foo = Foo> + Send + Sync> {
    fn trigger_on_inconsistent_topic(
        &mut self,
        the_topic: ListenerTopicNode,
        status: InconsistentTopicStatus,
    ) {
        self.on_inconsistent_topic(&Topic::new(TopicNodeKind::Listener(the_topic)), status)
    }
}
