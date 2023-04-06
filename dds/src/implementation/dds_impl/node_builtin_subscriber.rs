use crate::{
    implementation::utils::node::{ChildNode, RootNode},
    infrastructure::error::DdsResult,
    topic_definition::type_support::DdsType,
};

use super::{
    builtin_subscriber::BuiltInSubscriber, domain_participant_impl::DomainParticipantImpl,
    node_builtin_data_reader_stateful::BuiltinDataReaderStatefulNode,
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
    ) -> DdsResult<Option<BuiltinDataReaderStatefulNode>>
    where
        Foo: DdsType,
    {
        // let reader = match topic_name {
        //     "DCPSParticipant" if Foo::type_name() == ParticipantBuiltinTopicData::type_name() => {
        //         Ok(self.spdp_builtin_participant_reader.clone())
        //     }
        //     "DCPSTopic" if Foo::type_name() == TopicBuiltinTopicData::type_name() => {
        //         Ok(self.sedp_builtin_topics_reader.clone())
        //     }
        //     "DCPSPublication" if Foo::type_name() == PublicationBuiltinTopicData::type_name() => {
        //         Ok(self.sedp_builtin_publications_reader.clone())
        //     }
        //     "DCPSSubscription" if Foo::type_name() == SubscriptionBuiltinTopicData::type_name() => {
        //         Ok(self.sedp_builtin_subscriptions_reader.clone())
        //     }

        //     _ => Err(DdsError::BadParameter),
        // }

        // let reader = self.0.get()?.lookup_datareader::<Foo>(topic_name)?;

        // Ok(Some(BuiltinDataReaderNode::new(ChildNode::new(
        //     reader.downgrade(),
        //     self.0.clone(),
        // ))))
        todo!()
    }
}
