use crate::{
    infrastructure::{entity::Entity, qos::TopicQos},
    return_type::DDSResult,
};

use super::topic_listener::TopicListener;

/// TopicDescription represents the fact that both publications and subscriptions are tied to a single data-type. Its attribute
/// type_name defines a unique resulting type for the publication or the subscription and therefore creates an implicit association
/// with a TypeSupport. TopicDescription has also a name that allows it to be retrieved locally.
/// This class is an abstract class. It is the base class for Topic, ContentFilteredTopic, and MultiTopic.
pub trait TopicDescription<'a, T: 'a>:
    Entity<Qos = TopicQos<'a>, Listener = &'a (dyn TopicListener<DataType = T> + 'a)>
{
    /// The type_name used to create the TopicDescription
    fn get_type_name(&self) -> DDSResult<&'static str>;

    /// The name used to create the TopicDescription
    fn get_name(&self) -> DDSResult<&'a str>;
}

pub trait AnyTopicDescription {}
