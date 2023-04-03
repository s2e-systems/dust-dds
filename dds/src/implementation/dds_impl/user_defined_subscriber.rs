use crate::implementation::utils::node::ChildNode;

use super::{
    domain_participant_impl::DomainParticipantImpl,
    user_defined_subscriber_impl::UserDefinedSubscriberImpl,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedSubscriber(ChildNode<UserDefinedSubscriberImpl, DomainParticipantImpl>);

impl UserDefinedSubscriber {
    pub fn new(node: ChildNode<UserDefinedSubscriberImpl, DomainParticipantImpl>) -> Self {
        Self(node)
    }
}
