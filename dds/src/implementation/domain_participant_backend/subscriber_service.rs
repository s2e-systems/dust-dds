use std::collections::{HashMap, HashSet};

use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{BuiltInTopicKey, SubscriptionBuiltinTopicData, DCPS_PUBLICATION},
    dds_async::subscriber_listener::SubscriberListenerAsync,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        status_condition::status_condition_actor::StatusConditionActor,
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
    },
    rtps::{
        reader::{ReaderCacheChange, ReaderHistoryCache},
        types::TopicKind,
    },
    subscription::sample_info::{InstanceStateKind, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    topic_definition::type_support::DdsDeserialize,
    xtypes::dynamic_type::DynamicType,
};

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    data_reader::DataReaderActor,
    domain_participant_actor::{AddCacheChange, DomainParticipantActor},
};

pub struct CreateUserDefinedDataReader {
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataReaderQos>,
    pub a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
    pub mask: Vec<StatusKind>,
    pub domain_participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for CreateUserDefinedDataReader {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateUserDefinedDataReader> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: CreateUserDefinedDataReader,
    ) -> <CreateUserDefinedDataReader as Mail>::Result {
        struct UserDefinedReaderHistoryCache {
            pub domain_participant_address: ActorAddress<DomainParticipantActor>,
            pub subscriber_handle: InstanceHandle,
            pub data_reader_handle: InstanceHandle,
        }

        impl ReaderHistoryCache for UserDefinedReaderHistoryCache {
            fn add_change(&mut self, cache_change: ReaderCacheChange) {
                self.domain_participant_address
                    .send_actor_mail(AddCacheChange {
                        domain_participant_address: self.domain_participant_address.clone(),
                        cache_change,
                        subscriber_handle: self.subscriber_handle,
                        data_reader_handle: self.data_reader_handle,
                    })
                    .ok();
            }
        }

        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let topic = self
            .topic_list
            .get(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic_kind = get_topic_kind(topic.type_support().as_ref());
        let reader_handle = self.instance_handle_counter.generate_new_instance_handle();
        let transport_reader = self.transport.create_user_defined_reader(
            &message.topic_name,
            topic_kind,
            Box::new(UserDefinedReaderHistoryCache {
                domain_participant_address: message.domain_participant_address,
                subscriber_handle: subscriber.instance_handle(),
                data_reader_handle: reader_handle,
            }),
        );
        let qos = match message.qos {
            QosKind::Default => subscriber.default_data_reader_qos().clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let topic_name = topic.topic_name().to_owned();
        let type_name = topic.type_name().to_owned();

        let type_support = topic.type_support().clone();

        let data_reader_status_kind = message.mask.to_vec();
        let status_condition =
            Actor::spawn(StatusConditionActor::default(), &self.executor.handle());
        let data_reader_listener_thread = None;
        let mut data_reader = DataReaderActor::new(
            reader_handle,
            qos,
            topic_name,
            type_name,
            type_support,
            status_condition,
            data_reader_listener_thread,
            data_reader_status_kind,
            transport_reader,
        );

        let data_reader_handle = data_reader.instance_handle();
        let reader_status_condition_address = data_reader.status_condition().address();

        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            data_reader.enable();
        }

        subscriber.insert_data_reader(data_reader);

        if subscriber.enabled() && subscriber.qos().entity_factory.autoenable_created_entities {
            subscriber
                .get_mut_data_reader(data_reader_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .enable();

            if let Some(dcps_publication_reader) = self
                .builtin_subscriber
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == DCPS_PUBLICATION)
            {
                if let Ok(sample_list) = dcps_publication_reader.read(
                    i32::MAX,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    &[InstanceStateKind::Alive],
                    None,
                ) {
                    for (sample_data, _) in sample_list {
                        if let Ok(discovered_writer_data) = DiscoveredWriterData::deserialize_data(
                            sample_data
                                .expect("Alive samples should always contain data")
                                .as_ref(),
                        ) {
                            let is_any_name_matched = discovered_writer_data
                                .dds_publication_data
                                .partition
                                .name
                                .iter()
                                .any(|n| subscriber.qos().partition.name.contains(n));

                            let is_any_received_regex_matched_with_partition_qos =
                                discovered_writer_data
                                    .dds_publication_data
                                    .partition
                                    .name
                                    .iter()
                                    .filter_map(|n| glob_to_regex(n).ok())
                                    .any(|regex| {
                                        subscriber
                                            .qos()
                                            .partition
                                            .name
                                            .iter()
                                            .any(|n| regex.is_match(n))
                                    });

                            let is_any_local_regex_matched_with_received_partition_qos = subscriber
                                .qos()
                                .partition
                                .name
                                .iter()
                                .filter_map(|n| glob_to_regex(n).ok())
                                .any(|regex| {
                                    discovered_writer_data
                                        .dds_publication_data
                                        .partition
                                        .name
                                        .iter()
                                        .any(|n| regex.is_match(n))
                                });

                            let is_partition_matched =
                                discovered_writer_data.dds_publication_data.partition
                                    == subscriber.qos().partition
                                    || is_any_name_matched
                                    || is_any_received_regex_matched_with_partition_qos
                                    || is_any_local_regex_matched_with_received_partition_qos;
                            if is_partition_matched {
                                for dr in subscriber.data_reader_list_mut().filter(|dr| {
                                    dr.topic_name()
                                        == discovered_writer_data.dds_publication_data.topic_name
                                }) {
                                    todo!()
                                }
                            }
                        }
                    }
                }
            }

            let subscriber_qos = subscriber.qos().clone();
            let dr = subscriber
                .get_mut_data_reader(data_reader_handle)
                .ok_or(DdsError::AlreadyDeleted)?;
            let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: dr.transport_reader().guid(),
                },
                participant_key: BuiltInTopicKey { value: [0; 16] },
                topic_name: dr.topic_name().to_owned(),
                type_name: dr.type_name().to_owned(),
                durability: dr.qos().durability.clone(),
                deadline: dr.qos().deadline.clone(),
                latency_budget: dr.qos().latency_budget.clone(),
                liveliness: dr.qos().liveliness.clone(),
                reliability: dr.qos().reliability.clone(),
                ownership: dr.qos().ownership.clone(),
                destination_order: dr.qos().destination_order.clone(),
                user_data: dr.qos().user_data.clone(),
                time_based_filter: dr.qos().time_based_filter.clone(),
                presentation: subscriber_qos.presentation.clone(),
                partition: subscriber_qos.partition.clone(),
                topic_data: topic.qos().topic_data.clone(),
                group_data: subscriber_qos.group_data.clone(),
                representation: dr.qos().representation.clone(),
            };

            self.announce_created_or_modified_datareader(subscription_builtin_topic_data)?;
        }

        Ok((data_reader_handle, reader_status_condition_address))
    }
}

pub struct DeleteUserDefinedDataReader {
    pub subscriber_handle: InstanceHandle,
    pub datareader_handle: InstanceHandle,
}
impl Mail for DeleteUserDefinedDataReader {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteUserDefinedDataReader> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteUserDefinedDataReader,
    ) -> <DeleteUserDefinedDataReader as Mail>::Result {
        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|s| s.instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let dr = subscriber
            .remove_data_reader(message.datareader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic = &self.topic_list[dr.topic_name()];
        let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey {
                value: dr.transport_reader().guid(),
            },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: dr.topic_name().to_owned(),
            type_name: dr.type_name().to_owned(),
            durability: dr.qos().durability.clone(),
            deadline: dr.qos().deadline.clone(),
            latency_budget: dr.qos().latency_budget.clone(),
            liveliness: dr.qos().liveliness.clone(),
            reliability: dr.qos().reliability.clone(),
            ownership: dr.qos().ownership.clone(),
            destination_order: dr.qos().destination_order.clone(),
            user_data: dr.qos().user_data.clone(),
            time_based_filter: dr.qos().time_based_filter.clone(),
            presentation: subscriber.qos().presentation.clone(),
            partition: subscriber.qos().partition.clone(),
            topic_data: topic.qos().topic_data.clone(),
            group_data: subscriber.qos().group_data.clone(),
            representation: dr.qos().representation.clone(),
        };
        self.announce_deleted_data_reader(subscription_builtin_topic_data)?;
        Ok(())
    }
}

pub struct LookupDataReader {
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
}
impl Mail for LookupDataReader {
    type Result = DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>)>>;
}
impl MailHandler<LookupDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: LookupDataReader) -> <LookupDataReader as Mail>::Result {
        if !self.topic_list.contains_key(&message.topic_name) {
            return Err(DdsError::BadParameter);
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if self.get_instance_handle() == message.subscriber_handle {
            Ok(self
                .builtin_subscriber
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == message.topic_name)
                .map(|x: &mut DataReaderActor| {
                    (x.instance_handle(), x.status_condition().address())
                }))
        } else {
            let s = self
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle() == message.subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?;
            Ok(s.data_reader_list_mut()
                .find(|dr| dr.topic_name() == message.topic_name)
                .map(|x| (x.instance_handle(), x.status_condition().address())))
        }
    }
}

pub struct DeleteSubscriberContainedEntities {
    pub subscriber_handle: InstanceHandle,
}
impl Mail for DeleteSubscriberContainedEntities {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteSubscriberContainedEntities> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: DeleteSubscriberContainedEntities,
    ) -> <DeleteSubscriberContainedEntities as Mail>::Result {
        //         let deleted_reader_actor_list = self
        //         .subscriber_address
        //         .send_actor_mail(subscriber_actor::DrainDataReaderList)?
        //         .receive_reply()
        //         .await;

        //     for deleted_reader_actor in deleted_reader_actor_list {
        //         todo!();
        //         // self.announce_deleted_data_reader(&deleted_reader_actor, &topic)
        //         //     .await?;
        //         deleted_reader_actor.stop().await;
        //     }
        //     Ok(())
        // }
        todo!()
    }
}

pub struct SetDefaultDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub qos: QosKind<DataReaderQos>,
}
impl Mail for SetDefaultDataReaderQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultDataReaderQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetDefaultDataReaderQos,
    ) -> <SetDefaultDataReaderQos as Mail>::Result {
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

pub struct GetDefaultDataReaderQos {
    pub subscriber_handle: InstanceHandle,
}
impl Mail for GetDefaultDataReaderQos {
    type Result = DdsResult<DataReaderQos>;
}
impl MailHandler<GetDefaultDataReaderQos> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetDefaultDataReaderQos,
    ) -> <GetDefaultDataReaderQos as Mail>::Result {
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

pub struct SetSubscriberQos {
    pub subscriber_handle: InstanceHandle,
    pub qos: QosKind<SubscriberQos>,
}
impl Mail for SetSubscriberQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetSubscriberQos) -> <SetSubscriberQos as Mail>::Result {
        let subscriber = self
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => self.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };

        subscriber.set_qos(qos)
    }
}

pub struct GetSubscriberQos {
    pub subscriber_handle: InstanceHandle,
}
impl Mail for GetSubscriberQos {
    type Result = DdsResult<SubscriberQos>;
}
impl MailHandler<GetSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetSubscriberQos) -> <GetSubscriberQos as Mail>::Result {
        Ok(self
            .user_defined_subscriber_list
            .iter()
            .find(|s| s.instance_handle() == message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .qos()
            .clone())
    }
}

pub struct SetSubscriberListener {
    pub subscriber_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for SetSubscriberListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetSubscriberListener> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: SetSubscriberListener,
    ) -> <SetSubscriberListener as Mail>::Result {
        todo!()
    }
}

pub struct EnableSubscriber {
    pub subscriber_handle: InstanceHandle,
}
impl Mail for EnableSubscriber {
    type Result = DdsResult<()>;
}
impl MailHandler<EnableSubscriber> for DomainParticipantActor {
    fn handle(&mut self, message: EnableSubscriber) -> <EnableSubscriber as Mail>::Result {
        // if !self
        //     .subscriber_address
        //     .send_actor_mail(subscriber_actor::IsEnabled)?
        //     .receive_reply()
        //     .await
        // {
        //     self.subscriber_address
        //         .send_actor_mail(subscriber_actor::Enable)?
        //         .receive_reply()
        //         .await;

        //     if self
        //         .subscriber_address
        //         .send_actor_mail(subscriber_actor::GetQos)?
        //         .receive_reply()
        //         .await
        //         .entity_factory
        //         .autoenable_created_entities
        //     {
        //         for data_reader in self
        //             .subscriber_address
        //             .send_actor_mail(subscriber_actor::GetDataReaderList)?
        //             .receive_reply()
        //             .await
        //         {
        //             data_reader
        //                 .send_actor_mail(data_reader_actor::Enable {
        //                     data_reader_address: data_reader.clone(),
        //                 })?
        //                 .receive_reply()
        //                 .await;
        //         }
        //     }
        // }

        // Ok(())
        todo!()
    }
}

pub struct GetSubscriberInstanceHandle {
    pub subscriber_handle: InstanceHandle,
}
impl Mail for GetSubscriberInstanceHandle {
    type Result = DdsResult<InstanceHandle>;
}
impl MailHandler<GetSubscriberInstanceHandle> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: GetSubscriberInstanceHandle,
    ) -> <GetSubscriberInstanceHandle as Mail>::Result {
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
