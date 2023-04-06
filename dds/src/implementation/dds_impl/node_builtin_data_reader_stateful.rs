use crate::implementation::utils::node::{ChildNode, RootNode};

use super::{
    builtin_stateful_reader::BuiltinStatefulReader, builtin_subscriber::BuiltInSubscriber,
    domain_participant_impl::DomainParticipantImpl,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinDataReaderStatefulNode(
    ChildNode<BuiltinStatefulReader, ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>>,
);

impl BuiltinDataReaderStatefulNode {
    pub fn new(
        node: ChildNode<
            BuiltinStatefulReader,
            ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>,
        >,
    ) -> Self {
        Self(node)
    }
}
