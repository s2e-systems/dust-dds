use crate::{
    domain::domain_participant_factory::THE_DDS_DOMAIN_PARTICIPANT_FACTORY,
    implementation::{
        rtps::{
            endpoint::RtpsEndpoint,
            stateful_writer::RtpsStatefulWriter,
            types::{
                EntityId, EntityKey, Guid, TopicKind, USER_DEFINED_WRITER_NO_KEY,
                USER_DEFINED_WRITER_WITH_KEY,
            },
            writer::RtpsWriter,
        },
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind, TopicQos},
        status::StatusKind,
        time::{Duration, DURATION_ZERO},
    },
    publication::publisher_listener::PublisherListener,
    topic_definition::type_support::DdsType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, dds_data_writer::DdsDataWriter,
    dds_domain_participant::DdsDomainParticipant, node_domain_participant::DomainParticipantNode,
    node_user_defined_data_writer::UserDefinedDataWriterNode,
    status_condition_impl::StatusConditionImpl,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedPublisherNode {
    this: Guid,
    parent: Guid,
}

impl UserDefinedPublisherNode {
    pub fn new(this: Guid, parent: Guid) -> Self {
        Self { this, parent }
    }

    pub fn create_datawriter<Foo>(
        &self,
        type_name: &'static str,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedDataWriterNode>
    where
        Foo: DdsType,
    {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant_mut(&self.this.prefix(), |dp| {
            create_datawriter::<Foo>(
                dp.ok_or(DdsError::AlreadyDeleted)?,
                self.this,
                qos,
                a_listener,
                mask,
                type_name,
                topic_name,
            )
        })
    }

    pub fn delete_datawriter(&self, data_writer_handle: InstanceHandle) -> DdsResult<()> {
        todo!()
        // let data_writer = self
        //     .0
        //     .get()?
        //     .stateful_data_writer_list()
        //     .into_iter()
        //     .find(|x| InstanceHandle::from(x.guid()) == data_writer_handle)
        //     .ok_or_else(|| {
        //         DdsError::PreconditionNotMet(
        //             "Data writer can only be deleted from its parent publisher".to_string(),
        //         )
        //     })?
        //     .clone();

        // self.this
        //     .get()?
        //     .stateful_datawriter_delete(InstanceHandle::from(data_writer.guid()));

        // // The writer creation is announced only on enabled so its deletion must be announced only if it is enabled
        // if data_writer.is_enabled() {
        //     THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_dcps_service(self.this.parent(), |dp| {
        //         dp.unwrap()
        //             .announce_sender()
        //             .send(AnnounceKind::DeletedDataWriter(data_writer.guid().into()))
        //             .ok()
        //     });
        // }

        // Ok(())
    }

    pub fn lookup_datawriter(
        &self,
        type_name: &'static str,
        topic_name: &str,
    ) -> DdsResult<UserDefinedDataWriterNode> {
        todo!()
        // let writer = self
        //     .0
        //     .get()?
        //     .stateful_data_writer_list()
        //     .into_iter()
        //     .find(|data_writer| {
        //         data_writer.get_topic_name() == topic_name
        //             && data_writer.get_type_name() == type_name
        //     })
        //     .cloned()
        //     .ok_or_else(|| DdsError::PreconditionNotMet("Not found".to_string()))?;

        // Ok(UserDefinedDataWriterNode::new(ChildNode::new(
        //     writer.downgrade(),
        //     self.this.clone(),
        // )))
    }

    pub fn suspend_publications(&self) -> DdsResult<()> {
        todo!()
    }

    pub fn resume_publications(&self) -> DdsResult<()> {
        todo!()
    }

    pub fn begin_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    pub fn end_coherent_changes(&self) -> DdsResult<()> {
        todo!()
    }

    pub fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DdsResult<()> {
        todo!()
    }

    pub fn get_participant(&self) -> DdsResult<DomainParticipantNode> {
        Ok(DomainParticipantNode::new(self.parent))
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
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

    pub fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        todo!()
        // self.this.get()?.set_default_datawriter_qos(qos)
    }

    pub fn get_default_datawriter_qos(&self) -> DdsResult<DataWriterQos> {
        todo!()
        // Ok(self.this.get()?.get_default_datawriter_qos())
    }

    pub fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn set_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        todo!()
        // self.this.get()?.set_qos(qos)
    }

    pub fn get_qos(&self) -> DdsResult<PublisherQos> {
        todo!()
        // Ok(self.this.get()?.get_qos())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
        // *self.this.get()?.get_status_listener_lock() = StatusListener::new(a_listener, mask);
        // Ok(())
    }

    pub fn get_statuscondition(&self) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        // Ok(self.this.get()?.get_statuscondition())
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        // Ok(self.this.get()?.get_status_changes())
        todo!()
    }

    pub fn enable(&self) -> DdsResult<()> {
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

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        todo!()
        // Ok(InstanceHandle::from(self.this.get()?.guid()))
    }
}

fn create_datawriter<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    publisher_guid: Guid,
    qos: QosKind<DataWriterQos>,
    a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
    mask: &[StatusKind],
    type_name: &'static str,
    topic_name: String,
) -> DdsResult<UserDefinedDataWriterNode>
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

    let data_writer = DdsDataWriter::new(rtps_writer_impl, a_listener, mask, type_name, topic_name);

    publisher.stateful_datawriter_add(data_writer.clone());

    let data_writer_node =
        UserDefinedDataWriterNode::new(data_writer.guid(), publisher_guid, domain_participant_guid);

    if publisher.is_enabled()
        && publisher
            .get_qos()
            .entity_factory
            .autoenable_created_entities
    {
        data_writer_node.enable(domain_participant)?;
    }

    Ok(data_writer_node)
}
