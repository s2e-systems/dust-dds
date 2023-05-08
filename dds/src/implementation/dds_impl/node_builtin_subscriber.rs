use crate::{
    implementation::rtps::types::Guid,
    infrastructure::{
        condition::StatusCondition, error::DdsResult, instance::InstanceHandle, qos::SubscriberQos,
        status::StatusKind,
    },
    topic_definition::type_support::DdsType,
};

use super::{
    dds_domain_participant::DdsDomainParticipant,
    node_builtin_data_reader_stateful::BuiltinDataReaderStatefulNode,
    node_builtin_data_reader_stateless::BuiltinDataReaderStatelessNode,
    node_kind::DataReaderNodeKind,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinSubscriberNode {
    this: Guid,
    parent: Guid,
}

impl BuiltinSubscriberNode {
    pub fn new(this: Guid, parent: Guid) -> Self {
        Self { this, parent }
    }

    pub fn guid(&self) -> DdsResult<Guid> {
        Ok(self.this)
    }

    pub fn lookup_datareader<Foo>(
        &self,
        domain_participant: &DdsDomainParticipant,
        topic_name: &str,
    ) -> DdsResult<Option<DataReaderNodeKind>>
    where
        Foo: DdsType,
    {
        if let Some(r) = domain_participant
            .get_builtin_subscriber()
            .stateful_data_reader_list()
            .into_iter()
            .find(|x| x.get_type_name() == Foo::type_name() && x.get_topic_name() == topic_name)
        {
            Ok(Some(DataReaderNodeKind::BuiltinStateful(
                BuiltinDataReaderStatefulNode::new(r.guid(), self.this, self.parent),
            )))
        } else if let Some(r) = domain_participant
            .get_builtin_subscriber()
            .stateless_data_reader_list()
            .into_iter()
            .find(|x| x.get_type_name() == Foo::type_name() && x.get_topic_name() == topic_name)
        {
            Ok(Some(DataReaderNodeKind::BuiltinStateless(
                BuiltinDataReaderStatelessNode::new(r.guid(), self.this, self.parent),
            )))
        } else {
            Ok(None)
        }
    }

    pub fn get_qos(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<SubscriberQos> {
        Ok(domain_participant.get_builtin_subscriber().get_qos())
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.this.into())
    }
}
