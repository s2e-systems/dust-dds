use std::collections::{HashMap, HashSet};

use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, DCPS_SUBSCRIPTION},
    dds_async::publisher_listener::PublisherListenerAsync,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        actors::status_condition_actor::StatusConditionActor,
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::{
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            StatusKind,
        },
    },
    rtps::types::TopicKind,
    subscription::sample_info::{InstanceStateKind, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::DdsDeserialize,
    xtypes::dynamic_type::DynamicType,
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    data_writer::{DataWriterActor, DataWriterListenerThread},
    domain_participant_actor::DomainParticipantActor,
    publisher_listener::PublisherListenerThread,
};

pub struct CreateUserDefinedDataWriter {
    pub publisher_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataWriterQos>,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for CreateUserDefinedDataWriter {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedDataWriter> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedDataWriter,
    ) -> <CreateUserDefinedDataWriter as Mail>::Result {
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
        let mut data_writer = DataWriterActor {
            instance_handle: writer_handle,
            transport_writer,
            topic_name,
            type_name,
            matched_subscription_list: HashMap::new(),
            publication_matched_status: PublicationMatchedStatus::default(),
            incompatible_subscription_list: HashSet::new(),
            offered_incompatible_qos_status: OfferedIncompatibleQosStatus::default(),
            enabled: false,
            status_condition: Actor::spawn(
                StatusConditionActor::default(),
                &self.executor.handle(),
            ),
            data_writer_listener_thread: message.a_listener.map(DataWriterListenerThread::new),
            status_kind: message.mask,
            max_seq_num: None,
            last_change_sequence_number: 0,
            qos,
            registered_instance_list: HashSet::new(),
            offered_deadline_missed_status: OfferedDeadlineMissedStatus::default(),
            instance_deadline_missed_task: HashMap::new(),
            instance_samples: HashMap::new(),
        };
        let data_writer_handle = data_writer.instance_handle;
        let writer_status_condition_address = data_writer.status_condition.address();
        if publisher.enabled() && publisher.qos().entity_factory.autoenable_created_entities {
            data_writer.enabled = true;
        }

        publisher.insert_data_writer(data_writer);

        if publisher.enabled() && publisher.qos().entity_factory.autoenable_created_entities {
            publisher
                .data_writer_list_mut()
                .find(|x| x.instance_handle == data_writer_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .enabled = true;

            if let Some(dcps_subscription_reader) = self
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| dr.topic_name == DCPS_SUBSCRIPTION)
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
                                    dw.topic_name
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
                    value: data_writer.transport_writer.guid(),
                },
                participant_key: BuiltInTopicKey { value: [0; 16] },
                topic_name: data_writer.topic_name.clone(),
                type_name: data_writer.type_name.clone(),
                durability: data_writer.qos.durability.clone(),
                deadline: data_writer.qos.deadline.clone(),
                latency_budget: data_writer.qos.latency_budget.clone(),
                liveliness: data_writer.qos.liveliness.clone(),
                reliability: data_writer.qos.reliability.clone(),
                lifespan: data_writer.qos.lifespan.clone(),
                user_data: data_writer.qos.user_data.clone(),
                ownership: data_writer.qos.ownership.clone(),
                ownership_strength: data_writer.qos.ownership_strength.clone(),
                destination_order: data_writer.qos.destination_order.clone(),
                presentation: publisher.qos().presentation.clone(),
                partition: publisher.qos().partition.clone(),
                topic_data: self.topic_list[&data_writer.topic_name]
                    .qos()
                    .topic_data
                    .clone(),
                group_data: publisher.qos().group_data.clone(),
                representation: data_writer.qos.representation.clone(),
            };

            self.announce_created_or_modified_datawriter(publication_builtin_topic_data)?;
        }

        Ok((data_writer_handle.into(), writer_status_condition_address))
    }
}

pub struct DeleteUserDefinedDataWriter {
    pub publisher_handle: InstanceHandle,
    pub datawriter_handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedDataWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedDataWriter> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedDataWriter,
    ) -> <DeleteUserDefinedDataWriter as Mail>::Result {
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
                value: data_writer.transport_writer.guid(),
            },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: data_writer.topic_name.clone(),
            type_name: data_writer.type_name.clone(),
            durability: data_writer.qos.durability.clone(),
            deadline: data_writer.qos.deadline.clone(),
            latency_budget: data_writer.qos.latency_budget.clone(),
            liveliness: data_writer.qos.liveliness.clone(),
            reliability: data_writer.qos.reliability.clone(),
            lifespan: data_writer.qos.lifespan.clone(),
            user_data: data_writer.qos.user_data.clone(),
            ownership: data_writer.qos.ownership.clone(),
            ownership_strength: data_writer.qos.ownership_strength.clone(),
            destination_order: data_writer.qos.destination_order.clone(),
            presentation: publisher.qos().presentation.clone(),
            partition: publisher.qos().partition.clone(),
            topic_data: self.topic_list[&data_writer.topic_name]
                .qos()
                .topic_data
                .clone(),
            group_data: publisher.qos().group_data.clone(),
            representation: data_writer.qos.representation.clone(),
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

pub struct DeletePublisherContainedEntities {
    pub publisher_handle: InstanceHandle,
}
impl Mail for DeletePublisherContainedEntities {
    type Result = DdsResult<()>;
}
impl MailHandler<DeletePublisherContainedEntities> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeletePublisherContainedEntities,
    ) -> <DeletePublisherContainedEntities as Mail>::Result {
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

pub struct SetPublisherQos {
    pub publisher_handle: InstanceHandle,
    pub qos: QosKind<PublisherQos>,
}
impl Mail for SetPublisherQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetPublisherQos) -> <SetPublisherQos as Mail>::Result {
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

pub struct GetPublisherQos {
    pub publisher_handle: InstanceHandle,
}
impl Mail for GetPublisherQos {
    type Result = DdsResult<PublisherQos>;
}
impl MailHandler<GetPublisherQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetPublisherQos) -> <GetPublisherQos as Mail>::Result {
        Ok(self
            .user_defined_publisher_list
            .iter()
            .find(|p| p.instance_handle() == message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .qos()
            .clone())
    }
}

pub struct SetPublisherListener {
    pub publisher_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for SetPublisherListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetPublisherListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetPublisherListener) -> <SetPublisherQos as Mail>::Result {
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

pub struct EnablePublisher {
    pub publisher_handle: InstanceHandle,
}
impl Mail for EnablePublisher {
    type Result = DdsResult<()>;
}
impl MailHandler<EnablePublisher> for DomainParticipantActor {
    fn handle(&mut self, message: EnablePublisher) -> <EnablePublisher as Mail>::Result {
        todo!()
    }
}

pub struct GetPublisherInstanceHandle {
    pub publisher_handle: InstanceHandle,
}
impl Mail for GetPublisherInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetPublisherInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetPublisherInstanceHandle,
    ) -> <GetPublisherInstanceHandle as Mail>::Result {
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
