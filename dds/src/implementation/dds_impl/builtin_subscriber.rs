use crate::{
    implementation::utils::node::ChildNode, infrastructure::error::DdsResult,
    topic_definition::type_support::DdsType,
};

use super::{
    builtin_subscriber_impl::{BuiltInSubscriberImpl, BuiltinDataReaderKind},
    domain_participant_impl::DomainParticipantImpl,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinSubscriber(ChildNode<BuiltInSubscriberImpl, DomainParticipantImpl>);

impl BuiltinSubscriber {
    pub fn new(node: ChildNode<BuiltInSubscriberImpl, DomainParticipantImpl>) -> Self {
        Self(node)
    }

    pub fn lookup_datareader<Foo>(&self, topic_name: &str) -> DdsResult<BuiltinDataReaderKind>
    where
        Foo: DdsType,
    {
        self.0.get()?.lookup_datareader::<Foo>(topic_name)
    }
}
