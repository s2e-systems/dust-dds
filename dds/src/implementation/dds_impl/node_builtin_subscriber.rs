use crate::{
    implementation::utils::node::{ChildNode, RootNode},
    infrastructure::error::DdsResult,
    topic_definition::type_support::DdsType,
};

use super::{
    builtin_subscriber::{BuiltInSubscriber, BuiltinDataReaderKind},
    domain_participant_impl::DomainParticipantImpl,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinSubscriberNode(ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>);

impl BuiltinSubscriberNode {
    pub fn new(node: ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>) -> Self {
        Self(node)
    }

    pub fn lookup_datareader<Foo>(&self, topic_name: &str) -> DdsResult<BuiltinDataReaderKind>
    where
        Foo: DdsType,
    {
        self.0.get()?.lookup_datareader::<Foo>(topic_name)
    }
}
