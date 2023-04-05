use crate::{
    implementation::utils::node::{ChildNode, RootNode},
    infrastructure::error::DdsResult,
    topic_definition::type_support::DdsType,
};

use super::{
    builtin_subscriber::BuiltInSubscriber, domain_participant_impl::DomainParticipantImpl,
    node_builtin_data_reader::BuiltinDataReaderNode,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinSubscriberNode(ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>);

impl BuiltinSubscriberNode {
    pub fn new(node: ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>) -> Self {
        Self(node)
    }

    pub fn lookup_datareader<Foo>(
        &self,
        topic_name: &str,
    ) -> DdsResult<Option<BuiltinDataReaderNode>>
    where
        Foo: DdsType,
    {
        self.0.get()?.lookup_datareader::<Foo>(topic_name)?;

        Ok(Some(BuiltinDataReaderNode))
    }
}
