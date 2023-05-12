use crate::{
    implementation::rtps::{
        endpoint::RtpsEndpoint,
        stateful_writer::RtpsStatefulWriter,
        types::{
            EntityId, EntityKey, Guid, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
        writer::RtpsWriter,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        time::{Duration, DURATION_ZERO},
    },
    topic_definition::type_support::DdsType,
};

use super::{
    dds_data_writer::DdsDataWriter,
    dds_domain_participant::{AnnounceKind, DdsDomainParticipant},
    node_domain_participant::DomainParticipantNode,
    nodes::DataWriterNode,
};

#[derive(Eq, PartialEq, Debug)]
pub struct UserDefinedPublisherNode {
    this: Guid,
    parent: Guid,
}

impl UserDefinedPublisherNode {
    pub fn new(this: Guid, parent: Guid) -> Self {
        Self { this, parent }
    }

    pub fn guid(&self) -> Guid {
        self.this
    }

    pub fn parent_participant(&self) -> Guid {
        self.parent
    }
}

pub fn create_datawriter<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    publisher_guid: Guid,
    type_name: &'static str,
    topic_name: String,
    qos: QosKind<DataWriterQos>,
) -> DdsResult<DataWriterNode>
where
    Foo: DdsType,
{
    let default_unicast_locator_list = domain_participant.default_unicast_locator_list().to_vec();
    let default_multicast_locator_list =
        domain_participant.default_multicast_locator_list().to_vec();
    let data_max_size_serialized = domain_participant.data_max_size_serialized();
    let domain_participant_guid = domain_participant.guid();

    let publisher = domain_participant
        .user_defined_publisher_list_mut()
        .iter_mut()
        .find(|p| p.guid() == publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?;
    let qos = match qos {
        QosKind::Default => publisher.get_default_datawriter_qos(),
        QosKind::Specific(q) => q,
    };
    qos.is_consistent()?;

    let entity_kind = match Foo::has_key() {
        true => USER_DEFINED_WRITER_WITH_KEY,
        false => USER_DEFINED_WRITER_NO_KEY,
    };

    let entity_key = EntityKey::new([
        <[u8; 3]>::from(publisher.guid().entity_id().entity_key())[0],
        publisher.get_unique_writer_id(),
        0,
    ]);

    let entity_id = EntityId::new(entity_key, entity_kind);

    let guid = Guid::new(publisher.guid().prefix(), entity_id);

    let topic_kind = match Foo::has_key() {
        true => TopicKind::WithKey,
        false => TopicKind::NoKey,
    };

    let rtps_writer_impl = RtpsStatefulWriter::new(RtpsWriter::new(
        RtpsEndpoint::new(
            guid,
            topic_kind,
            &default_unicast_locator_list,
            &default_multicast_locator_list,
        ),
        true,
        Duration::new(0, 200_000_000),
        DURATION_ZERO,
        DURATION_ZERO,
        data_max_size_serialized,
        qos,
    ));

    let data_writer = DdsDataWriter::new(rtps_writer_impl, type_name, topic_name);

    let data_writer_node =
        DataWriterNode::new(data_writer.guid(), publisher_guid, domain_participant_guid);

    publisher.stateful_datawriter_add(data_writer);

    if publisher.is_enabled()
        && publisher
            .get_qos()
            .entity_factory
            .autoenable_created_entities
    {
        super::behavior_user_defined_data_writer::enable(domain_participant, guid, publisher_guid)?;
    }

    Ok(data_writer_node)
}

pub fn delete_datawriter(
    domain_participant: &mut DdsDomainParticipant,
    publisher_guid: Guid,
    data_writer_guid: Guid,
    data_writer_parent_publisher_guid: Guid,
) -> DdsResult<()> {
    if publisher_guid != data_writer_parent_publisher_guid {
        return Err(DdsError::PreconditionNotMet(
            "Data writer can only be deleted from its parent publisher".to_string(),
        ));
    }

    let data_writer_guid = domain_participant
        .user_defined_publisher_list_mut()
        .iter_mut()
        .find(|p| p.guid() == publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_data_writer_list()
        .iter()
        .find(|x| x.guid() == data_writer_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .guid();

    // The writer creation is announced only on enabled so its deletion must be announced only if it is enabled
    if domain_participant
        .user_defined_publisher_list_mut()
        .iter_mut()
        .find(|p| p.guid() == publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_data_writer_list()
        .iter()
        .find(|x| x.guid() == data_writer_guid)
        .ok_or_else(|| {
            DdsError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            )
        })?
        .is_enabled()
    {
        domain_participant
            .announce_sender()
            .send(AnnounceKind::DeletedDataWriter(data_writer_guid.into()))
            .ok();
    }

    domain_participant
        .user_defined_publisher_list_mut()
        .iter_mut()
        .find(|p| p.guid() == publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_datawriter_delete(InstanceHandle::from(data_writer_guid));

    Ok(())
}

pub fn lookup_datawriter(
    domain_participant: &DdsDomainParticipant,
    publisher_guid: Guid,
    type_name: &'static str,
    topic_name: &str,
) -> DdsResult<Option<DataWriterNode>> {
    Ok(domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .stateful_data_writer_list()
        .iter()
        .find(|data_reader| {
            data_reader.get_topic_name() == topic_name && data_reader.get_type_name() == type_name
        })
        .map(|x| DataWriterNode::new(x.guid(), publisher_guid, domain_participant.guid())))
}

pub fn get_participant(participant_guid: Guid) -> DdsResult<DomainParticipantNode> {
    Ok(DomainParticipantNode::new(participant_guid))
}

pub fn delete_contained_entities() -> DdsResult<()> {
    todo!()
    // for data_writer in self.this.get()?.stateful_datawriter_drain().into_iter() {
    //     if data_writer.is_enabled() {
    //         THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_dcps_service(self.this.parent(), |dp| {
    //             dp.unwrap()
    //                 .announce_sender()
    //                 .send(AnnounceKind::DeletedDataWriter(data_writer.guid().into()))
    //                 .ok()
    //         });
    //     }
    // }

    // Ok(())
}

pub fn set_default_datawriter_qos(
    domain_participant: &mut DdsDomainParticipant,
    publisher_guid: Guid,
    qos: QosKind<DataWriterQos>,
) -> DdsResult<()> {
    domain_participant
        .get_publisher_mut(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .set_default_datawriter_qos(qos)
}

pub fn get_default_datawriter_qos(
    domain_participant: &DdsDomainParticipant,
    publisher_guid: Guid,
) -> DdsResult<DataWriterQos> {
    Ok(domain_participant
        .get_publisher(publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_default_datawriter_qos())
}

pub fn get_qos(
    domain_participant: &DdsDomainParticipant,
    publisher_guid: Guid,
) -> DdsResult<PublisherQos> {
    Ok(domain_participant
        .user_defined_publisher_list()
        .iter()
        .find(|p| p.guid() == publisher_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_qos())
}

pub fn enable() -> DdsResult<()> {
    // let is_parent_enabled = THE_DDS_DOMAIN_PARTICIPANT_FACTORY
    //     .get_participant(self.this.parent(), |dp| dp.unwrap().is_enabled());
    // if !is_parent_enabled {
    //     return Err(DdsError::PreconditionNotMet(
    //         "Parent participant is disabled".to_string(),
    //     ));
    // }

    // if !self.this.get()?.is_enabled() {
    //     self.this.get()?.enable();

    //     if self
    //         .0
    //         .get()?
    //         .get_qos()
    //         .entity_factory
    //         .autoenable_created_entities
    //     {
    //         for data_writer in &self.this.get()?.stateful_data_writer_list() {
    //             data_writer.enable();
    //             let topic =
    //                 THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(self.this.parent(), |dp| {
    //                     dp.unwrap()
    //                         .topic_list()
    //                         .into_iter()
    //                         .find(|t| {
    //                             t.get_name() == data_writer.get_topic_name()
    //                                 && t.get_type_name() == data_writer.get_type_name()
    //                         })
    //                         .cloned()
    //                         .expect("Topic must exist")
    //                 });

    //             THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_dcps_service(self.this.parent(), |dcps| {
    //                 let discovered_writer_data = data_writer.as_discovered_writer_data(
    //                     &topic.get_qos(),
    //                     &self.this.get().unwrap().get_qos(),
    //                 );
    //                 dcps.unwrap()
    //                     .announce_sender()
    //                     .send(AnnounceKind::CreatedDataWriter(discovered_writer_data))
    //                     .ok()
    //             });
    //         }
    //     }
    // }

    // Ok(())
    todo!()
}

pub fn get_instance_handle(publisher_guid: Guid) -> DdsResult<InstanceHandle> {
    Ok(InstanceHandle::from(publisher_guid))
}
