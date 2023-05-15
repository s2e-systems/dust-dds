use crate::{
    implementation::{
        dds::{
            dds_domain_participant::DdsDomainParticipant,
            nodes::{DataReaderNode, DataReaderNodeKind},
        },
        rtps::types::Guid,
    },
    infrastructure::{error::DdsResult, qos::SubscriberQos},
    topic_definition::type_support::DdsType,
};

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
