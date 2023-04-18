use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    implementation::utils::node::{ChildNode, RootNode},
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::SubscriberQos,
        status::StatusKind,
    },
    topic_definition::type_support::DdsType,
};

use super::{
    builtin_subscriber::BuiltInSubscriber, dcps_service::DcpsService,
    domain_participant_impl::DomainParticipantImpl,
    node_builtin_data_reader_stateful::BuiltinDataReaderStatefulNode,
    node_builtin_data_reader_stateless::BuiltinDataReaderStatelessNode,
    node_kind::DataReaderNodeKind,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinSubscriberNode(
    ChildNode<BuiltInSubscriber, ChildNode<DomainParticipantImpl, RootNode<DcpsService>>>,
);

impl BuiltinSubscriberNode {
    pub fn new(
        node: ChildNode<BuiltInSubscriber, ChildNode<DomainParticipantImpl, RootNode<DcpsService>>>,
    ) -> Self {
        Self(node)
    }

    pub fn lookup_datareader<Foo>(&self, topic_name: &str) -> DdsResult<Option<DataReaderNodeKind>>
    where
        Foo: DdsType,
    {
        match topic_name {
            "DCPSParticipant" if Foo::type_name() == ParticipantBuiltinTopicData::type_name() => {
                Ok(Some(DataReaderNodeKind::BuiltinStateless(
                    BuiltinDataReaderStatelessNode::new(ChildNode::new(
                        self.0.get()?.spdp_builtin_participant_reader().downgrade(),
                        self.0.clone(),
                    )),
                )))
            }
            "DCPSTopic" if Foo::type_name() == TopicBuiltinTopicData::type_name() => {
                Ok(Some(DataReaderNodeKind::BuiltinStateful(
                    BuiltinDataReaderStatefulNode::new(ChildNode::new(
                        self.0.get()?.sedp_builtin_topics_reader().downgrade(),
                        self.0.clone(),
                    )),
                )))
            }
            "DCPSPublication" if Foo::type_name() == PublicationBuiltinTopicData::type_name() => {
                Ok(Some(DataReaderNodeKind::BuiltinStateful(
                    BuiltinDataReaderStatefulNode::new(ChildNode::new(
                        self.0.get()?.sedp_builtin_publications_reader().downgrade(),
                        self.0.clone(),
                    )),
                )))
            }
            "DCPSSubscription" if Foo::type_name() == SubscriptionBuiltinTopicData::type_name() => {
                Ok(Some(DataReaderNodeKind::BuiltinStateful(
                    BuiltinDataReaderStatefulNode::new(ChildNode::new(
                        self.0
                            .get()?
                            .sedp_builtin_subscriptions_reader()
                            .downgrade(),
                        self.0.clone(),
                    )),
                )))
            }

            _ => Err(DdsError::BadParameter),
        }
    }

    pub fn get_qos(&self) -> DdsResult<SubscriberQos> {
        Ok(self.0.get()?.get_qos())
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.0.get()?.get_statuscondition()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        self.0.get()?.get_status_changes()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }
}
