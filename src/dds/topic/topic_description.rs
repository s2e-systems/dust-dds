use crate::dds::domain::domain_participant::DomainParticipant;

/// TopicDescription represents the fact that both publications and subscriptions are tied to a single data-type. Its attribute
/// type_name defines a unique resulting type for the publication or the subscription and therefore creates an implicit association
/// with a TypeSupport. TopicDescription has also a name that allows it to be retrieved locally.
/// This class is an abstract class. It is the base class for Topic, ContentFilteredTopic, and MultiTopic.
pub struct TopicDescription{
    name: String,
    type_name: String,
}


impl TopicDescription {
    /// This operation returns the DomainParticipant to which the TopicDescription belongs.
    pub fn get_participant(&self) -> DomainParticipant {
        todo!()
    }    

    /// The type_name used to create the TopicDescription
    pub fn get_type_name(&self) -> &String {
        &self.name
    }

    /// The name used to create the TopicDescription
    pub fn get_name(&self) -> &String {
        &self.name
    }
}