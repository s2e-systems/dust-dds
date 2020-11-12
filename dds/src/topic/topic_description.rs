use crate::domain::DomainParticipant;
use rust_dds_interface::types::ReturnCode;

/// TopicDescription represents the fact that both publications and subscriptions are tied to a single data-type. Its attribute
/// type_name defines a unique resulting type for the publication or the subscription and therefore creates an implicit association
/// with a TypeSupport. TopicDescription has also a name that allows it to be retrieved locally.
/// This class is an abstract class. It is the base class for Topic, ContentFilteredTopic, and MultiTopic.
pub trait TopicDescription {
    /// This operation returns the DomainParticipant to which the TopicDescription belongs.
    fn get_participant(&self) -> &DomainParticipant;

    /// The type_name used to create the TopicDescription
    fn get_type_name(&self) -> ReturnCode<&str>;

    /// The name used to create the TopicDescription
    fn get_name(&self) -> ReturnCode<String>;
}