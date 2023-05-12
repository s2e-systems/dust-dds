use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint,
        reader::RtpsReader,
        stateful_reader::RtpsStatefulReader,
        types::{
            EntityId, EntityKey, Guid, TopicKind, USER_DEFINED_READER_NO_KEY,
            USER_DEFINED_READER_WITH_KEY,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        time::DURATION_ZERO,
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    dds_data_reader::DdsDataReader,
    dds_domain_participant::{AnnounceKind, DdsDomainParticipant},
    nodes::{DataReaderNode, DomainParticipantNode},
};

pub fn create_datareader<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    subscriber_guid: Guid,
    type_name: &'static str,
    topic_name: String,
    qos: QosKind<DataReaderQos>,
) -> DdsResult<DataReaderNode>
where
    Foo: DdsType + for<'de> DdsDeserialize<'de>,
{
    let default_unicast_locator_list = domain_participant.default_unicast_locator_list().to_vec();
    let default_multicast_locator_list =
        domain_participant.default_multicast_locator_list().to_vec();

    let qos = match qos {
        QosKind::Default => domain_participant
            .get_subscriber(subscriber_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_default_datareader_qos(),
        QosKind::Specific(q) => q,
    };
    qos.is_consistent()?;

    let entity_kind = match Foo::has_key() {
        true => USER_DEFINED_READER_WITH_KEY,
        false => USER_DEFINED_READER_NO_KEY,
    };

    let entity_key = EntityKey::new([
        <[u8; 3]>::from(
            domain_participant
                .get_subscriber(subscriber_guid)
                .ok_or(DdsError::AlreadyDeleted)?
                .guid()
                .entity_id()
                .entity_key(),
        )[0],
        domain_participant
            .get_subscriber_mut(subscriber_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_unique_reader_id(),
        0,
    ]);

    let entity_id = EntityId::new(entity_key, entity_kind);

    let guid = Guid::new(
        domain_participant
            .get_subscriber(subscriber_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .guid()
            .prefix(),
        entity_id,
    );

    let topic_kind = match Foo::has_key() {
        true => TopicKind::WithKey,
        false => TopicKind::NoKey,
    };

    let rtps_reader = RtpsStatefulReader::new(RtpsReader::new::<Foo>(
        RtpsEndpoint::new(
            guid,
            topic_kind,
            &default_unicast_locator_list,
            &default_multicast_locator_list,
        ),
        DURATION_ZERO,
        DURATION_ZERO,
        false,
        qos,
    ));

    let data_reader = DdsDataReader::new(rtps_reader, type_name, topic_name);

    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_data_reader_add(data_reader);

    let node = DataReaderNode::new(guid, subscriber_guid, domain_participant.guid());

    if domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .is_enabled()
        && domain_participant
            .get_subscriber(subscriber_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos()
            .entity_factory
            .autoenable_created_entities
    {
        super::behavior_user_defined_data_reader::enable(
            domain_participant,
            guid,
            subscriber_guid,
        )?;
    }

    Ok(node)
}

pub fn delete_datareader(
    domain_participant: &mut DdsDomainParticipant,
    subscriber_guid: Guid,
    a_datareader_handle: InstanceHandle,
) -> DdsResult<()> {
    let data_reader = domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_data_reader_list()
        .iter()
        .find(|x| x.get_instance_handle() == a_datareader_handle)
        .ok_or_else(|| {
            DdsError::PreconditionNotMet(
                "Data reader can only be deleted from its parent subscriber".to_string(),
            )
        })?;

    if data_reader.is_enabled() {
        domain_participant
            .announce_sender()
            .send(AnnounceKind::DeletedDataReader(
                data_reader.get_instance_handle(),
            ))
            .ok();
    }

    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_data_reader_delete(a_datareader_handle);

    Ok(())
}

pub fn lookup_datareader(
    domain_participant: &DdsDomainParticipant,
    subscriber_guid: Guid,
    type_name: &str,
    topic_name: &str,
) -> DdsResult<Option<DataReaderNode>> {
    Ok(domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_data_reader_list()
        .iter()
        .find(|data_reader| {
            data_reader.get_topic_name() == topic_name && data_reader.get_type_name() == type_name
        })
        .map(|x| DataReaderNode::new(x.guid(), subscriber_guid, domain_participant.guid())))
}

pub fn get_participant(participant_guid: Guid) -> DdsResult<DomainParticipantNode> {
    Ok(DomainParticipantNode::new(participant_guid))
}

pub fn delete_contained_entities(
    domain_participant: &mut DdsDomainParticipant,
    subscriber_guid: Guid,
) -> DdsResult<()> {
    for data_reader in domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_data_reader_drain()
    {
        if data_reader.is_enabled() {
            todo!()
            // domain_participant
            //     .announce_sender()
            //     .send(AnnounceKind::DeletedDataReader(
            //         data_reader.get_instance_handle(),
            //     ))
            //     .ok();
        }
    }
    Ok(())
}

pub fn set_default_datareader_qos(
    domain_participant: &mut DdsDomainParticipant,
    subscriber_guid: Guid,
    qos: QosKind<DataReaderQos>,
) -> DdsResult<()> {
    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .set_default_datareader_qos(qos)
}

pub fn get_default_datareader_qos(
    domain_participant: &DdsDomainParticipant,
    subscriber_guid: Guid,
) -> DdsResult<DataReaderQos> {
    Ok(domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_default_datareader_qos())
}

pub fn set_qos(
    domain_participant: &mut DdsDomainParticipant,
    subscriber_guid: Guid,
    qos: QosKind<SubscriberQos>,
) -> DdsResult<()> {
    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .set_qos(qos)
}

pub fn get_qos(
    domain_participant: &DdsDomainParticipant,
    subscriber_guid: Guid,
) -> DdsResult<SubscriberQos> {
    Ok(domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_qos())
}

pub fn enable(
    domain_participant: &mut DdsDomainParticipant,
    subscriber_guid: Guid,
) -> DdsResult<()> {
    let is_parent_enabled = domain_participant.is_enabled();
    if !is_parent_enabled {
        return Err(DdsError::PreconditionNotMet(
            "Parent participant is disabled".to_string(),
        ));
    }

    if !domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .is_enabled()
    {
        domain_participant
            .get_subscriber_mut(subscriber_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .enable()?;

        if domain_participant
            .get_subscriber(subscriber_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos()
            .entity_factory
            .autoenable_created_entities
        {
            for data_reader in domain_participant
                .get_subscriber_mut(subscriber_guid)
                .ok_or(DdsError::AlreadyDeleted)?
                .stateful_data_reader_list_mut()
            {
                data_reader.enable()?;
                // let topic = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(
                //     &self.0.parent().prefix(),
                //     |dp| {
                //         dp.unwrap()
                //             .topic_list()
                //             .into_iter()
                //             .find(|t| {
                //                 t.get_name() == data_reader.get_topic_name()
                //                     && t.get_type_name() == data_reader.get_type_name()
                //             })
                //             .cloned()
                //             .expect("Topic must exist")
                //     },
                // );

                // THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_dcps_service(
                //     &self.0.parent().prefix(),
                //     |dcps| {
                //         let discovered_reader_data = data_reader.as_discovered_reader_data(
                //             &topic.get_qos(),
                //             &self.0.get().unwrap().get_qos(),
                //         );
                //         domain_participant
                //             .announce_sender()
                //             .send(AnnounceKind::CreatedDataReader(discovered_reader_data))
                //             .ok()
                //     },
                // );
            }
        }
    }

    Ok(())
}
