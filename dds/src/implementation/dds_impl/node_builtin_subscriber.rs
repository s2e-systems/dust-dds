use crate::{
    implementation::rtps::types::Guid,
    infrastructure::{error::DdsResult, instance::InstanceHandle, qos::SubscriberQos},
    topic_definition::type_support::DdsType,
};

use super::{
    dds_domain_participant::DdsDomainParticipant,
    nodes::{DataReaderNode, DataReaderNodeKind},
};

#[derive(PartialEq, Eq, Debug)]
pub struct BuiltinSubscriberNode {
    this: Guid,
    parent: Guid,
}

impl BuiltinSubscriberNode {
    pub fn new(this: Guid, parent: Guid) -> Self {
        Self { this, parent }
    }

    pub fn guid(&self) -> Guid {
        self.this
    }
}

pub fn lookup_datareader<Foo>(
    domain_participant: &DdsDomainParticipant,
    subscriber_guid: Guid,
    topic_name: &str,
) -> DdsResult<Option<DataReaderNodeKind>>
where
    Foo: DdsType,
{
    if let Some(r) = domain_participant
        .get_builtin_subscriber()
        .stateful_data_reader_list()
        .iter()
        .find(|x| x.get_type_name() == Foo::type_name() && x.get_topic_name() == topic_name)
    {
        Ok(Some(DataReaderNodeKind::BuiltinStateful(
            DataReaderNode::new(r.guid(), subscriber_guid, domain_participant.guid()),
        )))
    } else if let Some(r) = domain_participant
        .get_builtin_subscriber()
        .stateless_data_reader_list()
        .iter()
        .find(|x| x.get_type_name() == Foo::type_name() && x.get_topic_name() == topic_name)
    {
        Ok(Some(DataReaderNodeKind::BuiltinStateless(
            DataReaderNode::new(r.guid(), subscriber_guid, domain_participant.guid()),
        )))
    } else {
        Ok(None)
    }
}

pub fn get_qos(domain_participant: &DdsDomainParticipant) -> DdsResult<SubscriberQos> {
    Ok(domain_participant.get_builtin_subscriber().get_qos())
}

pub fn get_instance_handle(subscriber_guid: Guid) -> DdsResult<InstanceHandle> {
    Ok(subscriber_guid.into())
}
