use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, DCPS_SUBSCRIPTION},
    dds_async::publisher_listener::PublisherListenerAsync,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        status_condition::status_condition_actor::StatusConditionActor,
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::StatusKind,
    },
    rtps::types::TopicKind,
    subscription::sample_info::{InstanceStateKind, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::DdsDeserialize,
    xtypes::dynamic_type::DynamicType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, data_writer::DataWriterActor,
    data_writer_listener::DataWriterListenerThread,
    domain_participant_actor::DomainParticipantActor, publisher_listener::PublisherListenerThread,
};

pub struct CreateDataWriter {
    pub publisher_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataWriterQos>,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateDataWriter {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: CreateDataWriter) -> <CreateDataWriter as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle() == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic = self
            .topic_list
            .get(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic_kind = get_topic_kind(topic.type_support().as_ref());
        let type_support = topic.type_support().clone();

        let transport_writer = self
            .transport
            .create_user_defined_writer(&message.topic_name, topic_kind);
        let writer_handle = self.instance_handle_counter.generate_new_instance_handle();
        let qos = match message.qos {
            QosKind::Default => publisher.default_datawriter_qos().clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let topic_name = message.topic_name;

        let type_name = topic.type_name().to_owned();
        let status_condition =
            Actor::spawn(StatusConditionActor::default(), &self.executor.handle());
        let writer_status_condition_address = status_condition.address();
        let mut data_writer = DataWriterActor::new(
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
        if publisher.enabled() && publisher.qos().entity_factory.autoenable_created_entities {
            data_writer.enable();
        }

        publisher.insert_data_writer(data_writer);

        if publisher.enabled() && publisher.qos().entity_factory.autoenable_created_entities {
            if let Some(dcps_subscription_reader) = self
                .builtin_subscriber
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == DCPS_SUBSCRIPTION)
            {
                if let Ok(sample_list) = dcps_subscription_reader.read(
                    i32::MAX,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    &[InstanceStateKind::Alive],
                    None,
                ) {
                    for (sample_data, _) in sample_list {
                        if let Ok(discovered_reader_data) = DiscoveredReaderData::deserialize_data(
                            sample_data
                                .expect("Alive samples should always contain data")
                                .as_ref(),
                        ) {
                            let is_any_name_matched = discovered_reader_data
                                .dds_subscription_data
                                .partition
                                .name
                                .iter()
                                .any(|n| publisher.qos().partition.name.contains(n));

                            let is_any_received_regex_matched_with_partition_qos =
                                discovered_reader_data
                                    .dds_subscription_data
                                    .partition
                                    .name
                                    .iter()
                                    .filter_map(|n| glob_to_regex(n).ok())
                                    .any(|regex| {
                                        publisher
                                            .qos()
                                            .partition
                                            .name
                                            .iter()
                                            .any(|n| regex.is_match(n))
                                    });

                            let is_any_local_regex_matched_with_received_partition_qos = publisher
                                .qos()
                                .partition
                                .name
                                .iter()
                                .filter_map(|n| glob_to_regex(n).ok())
                                .any(|regex| {
                                    discovered_reader_data
                                        .dds_subscription_data
                                        .partition
                                        .name
                                        .iter()
                                        .any(|n| regex.is_match(n))
                                });

                            let is_partition_matched =
                                discovered_reader_data.dds_subscription_data.partition
                                    == publisher.qos().partition
                                    || is_any_name_matched
                                    || is_any_received_regex_matched_with_partition_qos
                                    || is_any_local_regex_matched_with_received_partition_qos;

                            if is_partition_matched {
                                for dw in publisher.data_writer_list_mut().filter(|dw| {
                                    dw.topic_name()
                                        == discovered_reader_data
                                            .subscription_builtin_topic_data()
                                            .topic_name()
                                }) {
                                    todo!()
                                    // dw.add_matched_reader(&discovered_reader_data, &publisher.qos);
                                }
                            }
                        }
                    }
                }
            }

            let data_writer = publisher
                .get_data_writer(data_writer_handle)
                .ok_or(DdsError::AlreadyDeleted)?;
            let publication_builtin_topic_data = PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: data_writer.transport_writer().guid(),
                },
                participant_key: BuiltInTopicKey { value: [0; 16] },
                topic_name: data_writer.topic_name().to_owned(),
                type_name: data_writer.type_name().to_owned(),
                durability: data_writer.qos().durability.clone(),
                deadline: data_writer.qos().deadline.clone(),
                latency_budget: data_writer.qos().latency_budget.clone(),
                liveliness: data_writer.qos().liveliness.clone(),
                reliability: data_writer.qos().reliability.clone(),
                lifespan: data_writer.qos().lifespan.clone(),
                user_data: data_writer.qos().user_data.clone(),
                ownership: data_writer.qos().ownership.clone(),
                ownership_strength: data_writer.qos().ownership_strength.clone(),
                destination_order: data_writer.qos().destination_order.clone(),
                presentation: publisher.qos().presentation.clone(),
                partition: publisher.qos().partition.clone(),
                topic_data: self.topic_list[data_writer.topic_name()]
                    .qos()
                    .topic_data
                    .clone(),
                group_data: publisher.qos().group_data.clone(),
                representation: data_writer.qos().representation.clone(),
            };

            self.announce_created_or_modified_datawriter(publication_builtin_topic_data)?;
        }

        Ok((data_writer_handle.into(), writer_status_condition_address))
    }
}

pub struct DeleteDataWriter {
    pub publisher_handle: InstanceHandle,
    pub datawriter_handle: InstanceHandle,
}
impl Mail for DeleteDataWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteDataWriter) -> <DeleteDataWriter as Mail>::Result {
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|p| p.instance_handle() == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let data_writer = publisher
            .remove_data_writer(message.datawriter_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let publication_builtin_topic_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey {
                value: data_writer.transport_writer().guid(),
            },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: data_writer.topic_name().to_owned(),
            type_name: data_writer.type_name().to_owned(),
            durability: data_writer.qos().durability.clone(),
            deadline: data_writer.qos().deadline.clone(),
            latency_budget: data_writer.qos().latency_budget.clone(),
            liveliness: data_writer.qos().liveliness.clone(),
            reliability: data_writer.qos().reliability.clone(),
            lifespan: data_writer.qos().lifespan.clone(),
            user_data: data_writer.qos().user_data.clone(),
            ownership: data_writer.qos().ownership.clone(),
            ownership_strength: data_writer.qos().ownership_strength.clone(),
            destination_order: data_writer.qos().destination_order.clone(),
            presentation: publisher.qos().presentation.clone(),
            partition: publisher.qos().partition.clone(),
            topic_data: self.topic_list[data_writer.topic_name()]
                .qos()
                .topic_data
                .clone(),
            group_data: publisher.qos().group_data.clone(),
            representation: data_writer.qos().representation.clone(),
        };
        self.announce_deleted_data_writer(publication_builtin_topic_data)?;
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
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle() == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => q,
        };
        publisher.set_default_datawriter_qos(qos);
        Ok(())
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
        // let qos = match qos {
        //     QosKind::Default => {
        //         self.publisher_address
        //             .send_actor_mail(publisher_actor::GetDefaultDatawriterQos)?
        //             .receive_reply()
        //             .await
        //     }
        //     QosKind::Specific(q) => {
        //         q.is_consistent()?;
        //         q
        //     }
        // };

        // self.publisher_address
        //     .send_actor_mail(publisher_actor::SetDefaultDatawriterQos { qos })?
        //     .receive_reply()
        //     .await;

        // Ok(())
        todo!()
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
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle() == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let qos = match message.qos {
            QosKind::Default => self.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };

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
            .user_defined_publisher_list
            .iter()
            .find(|p| p.instance_handle() == message.publisher_handle)
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
        let publisher = self
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle() == message.publisher_handle)
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

pub struct GetInstanceHandle {
    pub publisher_handle: InstanceHandle,
}
impl Mail for GetInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetInstanceHandle> for DomainParticipantActor {
    fn handle(&mut self, message: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
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
