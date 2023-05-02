use crate::{
    implementation::utils::node::{ChildNode, RootNode},
    infrastructure::{
        condition::StatusCondition, error::DdsResult, instance::InstanceHandle, qos::SubscriberQos,
        status::StatusKind,
    },
    topic_definition::type_support::DdsType,
};

use super::{
    dcps_service::DcpsService, dds_subscriber::DdsSubscriber,
    dds_domain_participant::DdsDomainParticipant,
    node_builtin_data_reader_stateful::BuiltinDataReaderStatefulNode,
    node_builtin_data_reader_stateless::BuiltinDataReaderStatelessNode,
    node_kind::DataReaderNodeKind,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinSubscriberNode(
    ChildNode<DdsSubscriber, ChildNode<DdsDomainParticipant, RootNode<DcpsService>>>,
);

impl BuiltinSubscriberNode {
    pub fn new(
        node: ChildNode<DdsSubscriber, ChildNode<DdsDomainParticipant, RootNode<DcpsService>>>,
    ) -> Self {
        Self(node)
    }

    pub fn lookup_datareader<Foo>(&self, topic_name: &str) -> DdsResult<Option<DataReaderNodeKind>>
    where
        Foo: DdsType,
    {
        if let Some(r) = self
            .0
            .get()?
            .stateful_data_reader_list()
            .into_iter()
            .find(|x| x.get_type_name() == Foo::type_name() && x.get_topic_name() == topic_name)
        {
            Ok(Some(DataReaderNodeKind::BuiltinStateful(
                BuiltinDataReaderStatefulNode::new(ChildNode::new(r.downgrade(), self.0.clone())),
            )))
        } else if let Some(r) = self
            .0
            .get()?
            .stateless_data_reader_list()
            .into_iter()
            .find(|x| x.get_type_name() == Foo::type_name() && x.get_topic_name() == topic_name)
        {
            Ok(Some(DataReaderNodeKind::BuiltinStateless(
                BuiltinDataReaderStatelessNode::new(ChildNode::new(r.downgrade(), self.0.clone())),
            )))
        } else {
            Ok(None)
        }
    }

    pub fn get_qos(&self) -> DdsResult<SubscriberQos> {
        Ok(self.0.get()?.get_qos())
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }
}
