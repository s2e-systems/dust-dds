use crate::{
    dds_async::publisher_listener::PublisherListenerAsync,
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::data_writer::DataWriterEntity,
        },
        listeners::{
            data_writer_listener::DataWriterListenerThread,
            publisher_listener::PublisherListenerThread,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::StatusKind,
    },
    rtps::types::TopicKind,
    runtime::actor::{Actor, ActorAddress, Mail, MailHandler},
    xtypes::dynamic_type::DynamicType,
};

use super::{data_writer_service, discovery_service};

pub struct CreateDataWriter {
    pub publisher_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataWriterQos>,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub mask: Vec<StatusKind>,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for CreateDataWriter {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: CreateDataWriter) -> <CreateDataWriter as Mail>::Result {
        let topic = self
            .domain_participant
            .get_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic_kind = get_topic_kind(topic.type_support().as_ref());
        let type_support = topic.type_support().clone();
        let type_name = topic.type_name().to_owned();

        let transport_writer = self
            .transport
            .create_user_defined_writer(&message.topic_name, topic_kind);
        let writer_handle = self.instance_handle_counter.generate_new_instance_handle();
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => publisher.default_datawriter_qos().clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let topic_name = message.topic_name;

        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let writer_status_condition_address = status_condition.address();

        let data_writer = DataWriterEntity::new(
            writer_handle,
            transport_writer,
            topic_name,
            type_name,
            type_support,
            status_condition,
            message.a_listener.map(DataWriterListenerThread::new),
            message.mask,
            qos,
        );
        let data_writer_handle = data_writer.instance_handle();

        publisher.insert_data_writer(data_writer);

        if publisher.enabled() && publisher.qos().entity_factory.autoenable_created_entities {
            message
                .participant_address
                .send_actor_mail(data_writer_service::Enable {
                    publisher_handle: message.publisher_handle,
                    data_writer_handle: writer_handle,
                    participant_address: message.participant_address.clone(),
                })
                .ok();
        }

        Ok((data_writer_handle.into(), writer_status_condition_address))
    }
}

pub struct DeleteDataWriter {
    pub publisher_handle: InstanceHandle,
    pub datawriter_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for DeleteDataWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteDataWriter) -> <DeleteDataWriter as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let data_writer = publisher
            .remove_data_writer(message.datawriter_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        message
            .participant_address
            .send_actor_mail(discovery_service::AnnounceDeletedDataWriter { data_writer })
            .ok();
        Ok(())
    }
}

pub struct LookupDataWriter {
    pub publisher_handle: InstanceHandle,
    pub topic_name: String,
}
impl Mail for LookupDataWriter {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<LookupDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: LookupDataWriter) -> <LookupDataWriter as Mail>::Result {
        todo!()
        // if let Some(_) = self
        //     .participant
        //     .participant_address()
        //     .send_actor_mail(domain_participant_actor::LookupTopicdescription {
        //         topic_name: topic_name.to_string(),
        //     })?
        //     .receive_reply()
        //     .await?
        // {
        //     let data_writer_list = self
        //         .publisher_address
        //         .send_actor_mail(publisher_actor::GetDataWriterList)?
        //         .receive_reply()
        //         .await;
        //     for dw in data_writer_list {
        //         if dw
        //             .send_actor_mail(data_writer_actor::GetTopicName)?
        //             .receive_reply()
        //             .await?
        //             == topic_name
        //         {
        //             let type_name = self
        //                 .participant_address()
        //                 .send_actor_mail(domain_participant_actor::GetTopicTypeName {
        //                     topic_name: topic_name.to_string(),
        //                 })?
        //                 .receive_reply()
        //                 .await?;
        //             let topic = TopicAsync::new(
        //                 type_name,
        //                 topic_name.to_string(),
        //                 self.participant.clone(),
        //             );
        //             let status_condition = dw
        //                 .send_actor_mail(data_writer_actor::GetStatuscondition)?
        //                 .receive_reply()
        //                 .await;
        //             return Ok(Some(DataWriterAsync::new(
        //                 dw.clone(),
        //                 status_condition,
        //                 self.clone(),
        //                 topic,
        //             )));
        //         }
        //     }
        //     Ok(None)
        // } else {
        //     Err(DdsError::BadParameter)
        // }
    }
}

pub struct DeleteContainedEntities {
    pub publisher_handle: InstanceHandle,
}
impl Mail for DeleteContainedEntities {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteContainedEntities> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteContainedEntities,
    ) -> <DeleteContainedEntities as Mail>::Result {
        // let deleted_writer_actor_list = self
        //     .publisher_address
        //     .send_actor_mail(publisher_actor::DrainDataWriterList)?
        //     .receive_reply()
        //     .await;

        // for deleted_writer_actor in deleted_writer_actor_list {
        //     todo!();
        //     // self.announce_deleted_data_writer(&deleted_writer_actor, &topic_address)
        //     //     .await?;
        //     deleted_writer_actor.stop().await;
        // }
        // Ok(())
        todo!()
    }
}

pub struct SetDefaultDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub qos: QosKind<DataWriterQos>,
}
impl Mail for SetDefaultDataWriterQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultDataWriterQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDefaultDataWriterQos,
    ) -> <SetDefaultDataWriterQos as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => q,
        };
        publisher.set_default_datawriter_qos(qos)
    }
}

pub struct GetDefaultDataWriterQos {
    pub publisher_handle: InstanceHandle,
}
impl Mail for GetDefaultDataWriterQos {
    type Result = DdsResult<DataWriterQos>;
}
impl MailHandler<GetDefaultDataWriterQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDefaultDataWriterQos,
    ) -> <GetDefaultDataWriterQos as Mail>::Result {
        Ok(self
            .domain_participant
            .get_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .default_datawriter_qos()
            .clone())
    }
}

pub struct SetQos {
    pub publisher_handle: InstanceHandle,
    pub qos: QosKind<PublisherQos>,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => self.domain_participant.default_publisher_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        publisher.set_qos(qos)
    }
}

pub struct GetQos {
    pub publisher_handle: InstanceHandle,
}
impl Mail for GetQos {
    type Result = DdsResult<PublisherQos>;
}
impl MailHandler<GetQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetQos) -> <GetQos as Mail>::Result {
        Ok(self
            .domain_participant
            .get_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .qos()
            .clone())
    }
}

pub struct SetListener {
    pub publisher_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) -> <SetQos as Mail>::Result {
        self.domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .set_listener(
                message.a_listener.map(PublisherListenerThread::new),
                message.mask,
            );

        Ok(())
    }
}

pub struct Enable {
    pub publisher_handle: InstanceHandle,
}
impl Mail for Enable {
    type Result = DdsResult<()>;
}
impl MailHandler<Enable> for DomainParticipantActor {
    fn handle(&mut self, message: Enable) -> <Enable as Mail>::Result {
        todo!()
    }
}

fn get_topic_kind(type_support: &dyn DynamicType) -> TopicKind {
    for index in 0..type_support.get_member_count() {
        if let Ok(m) = type_support.get_member_by_index(index) {
            if let Ok(d) = m.get_descriptor() {
                if d.is_key {
                    return TopicKind::WithKey;
                }
            }
        }
    }
    TopicKind::NoKey
}
