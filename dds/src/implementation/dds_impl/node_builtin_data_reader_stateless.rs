use crate::implementation::utils::node::{ChildNode, RootNode};

use super::{
    builtin_stateless_reader::BuiltinStatelessReader, builtin_subscriber::BuiltInSubscriber,
    domain_participant_impl::DomainParticipantImpl,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinDataReaderStatelessNode(
    ChildNode<
        BuiltinStatelessReader,
        ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>,
    >,
);

impl BuiltinDataReaderStatelessNode {
    pub fn new(
        node: ChildNode<
            BuiltinStatelessReader,
            ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>,
        >,
    ) -> Self {
        Self(node)
    }
}
