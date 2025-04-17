use crate::{
    dds_async::subscriber_listener::SubscriberListenerAsync,
    implementation::{
        any_data_reader_listener::AnyDataReaderListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::data_reader::{DataReaderEntity, TransportReaderKind},
            services::{data_reader_service, discovery_service, message_service},
        },
        listeners::{
            data_reader_listener::DataReaderListenerActor,
            subscriber_listener::SubscriberListenerActor,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        qos_policy::ReliabilityQosPolicyKind,
        status::StatusKind,
    },
    runtime::{
        actor::{Actor, ActorAddress, MailHandler},
        oneshot::OneshotSender,
    },
    transport::{
        history_cache::{CacheChange, HistoryCache},
        types::{
            EntityId, ReliabilityKind, TopicKind, USER_DEFINED_READER_NO_KEY,
            USER_DEFINED_READER_WITH_KEY,
        },
    },
    xtypes::dynamic_type::DynamicType,
};

pub struct CreateDataReader {
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataReaderQos>,
    pub a_listener: Option<Box<dyn AnyDataReaderListener>>,
    pub mask: Vec<StatusKind>,
    pub domain_participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender:
        OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
}
impl MailHandler<CreateDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: CreateDataReader) {
        struct UserDefinedReaderHistoryCache {
            pub domain_participant_address: ActorAddress<DomainParticipantActor>,
            pub subscriber_handle: InstanceHandle,
            pub data_reader_handle: InstanceHandle,
        }

        impl HistoryCache for UserDefinedReaderHistoryCache {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.domain_participant_address
                    .send_actor_mail(message_service::AddCacheChange {
                        participant_address: self.domain_participant_address.clone(),
                        cache_change,
                        subscriber_handle: self.subscriber_handle,
                        data_reader_handle: self.data_reader_handle,
                    })
                    .ok();
            }

            fn remove_change(&mut self, _sequence_number: i64) {
                todo!()
            }
        }

        let Some(topic) = self.domain_participant.get_topic(&message.topic_name) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let topic_kind = get_topic_kind(topic.type_support().as_ref());
        let topic_name = topic.topic_name().to_owned();
        let type_name = topic.type_name().to_owned();
        let reader_handle = self.instance_handle_counter.generate_new_instance_handle();

        let type_support = topic.type_support().clone();
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let qos = match message.qos {
            QosKind::Default => subscriber.default_data_reader_qos().clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    message.reply_sender.send(Err(DdsError::InconsistentPolicy));
                    return;
                }
            }
        };
        self.entity_counter += 1;

        let entity_kind = match topic_kind {
            TopicKind::NoKey => USER_DEFINED_READER_NO_KEY,
            TopicKind::WithKey => USER_DEFINED_READER_WITH_KEY,
        };
        let entity_id = EntityId::new(
            [
                0,
                self.entity_counter.to_le_bytes()[0],
                self.entity_counter.to_le_bytes()[1],
            ],
            entity_kind,
        );
        let reliablity_kind = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        };
        let transport_reader =
            TransportReaderKind::Stateful(self.transport.create_stateful_reader(
                entity_id,
                reliablity_kind,
                Box::new(UserDefinedReaderHistoryCache {
                    domain_participant_address: message.domain_participant_address.clone(),
                    subscriber_handle: subscriber.instance_handle(),
                    data_reader_handle: reader_handle,
                }),
            ));

        let listener_mask = message.mask.to_vec();
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let listener = message.a_listener.map(|l| {
            Actor::spawn(
                DataReaderListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let data_reader = DataReaderEntity::new(
            reader_handle,
            qos,
            topic_name,
            type_name,
            type_support,
            status_condition,
            listener,
            listener_mask,
            transport_reader,
        );

        let data_reader_handle = data_reader.instance_handle();
        let reader_status_condition_address = data_reader.status_condition().address();

        subscriber.insert_data_reader(data_reader);

        if subscriber.enabled() && subscriber.qos().entity_factory.autoenable_created_entities {
            message
                .domain_participant_address
                .send_actor_mail(data_reader_service::Enable {
                    subscriber_handle: message.subscriber_handle,
                    data_reader_handle: reader_handle,
                    participant_address: message.domain_participant_address.clone(),
                })
                .ok();
        }
        message
            .reply_sender
            .send(Ok((data_reader_handle, reader_status_condition_address)))
    }
}

pub struct DeleteDataReader {
    pub subscriber_handle: InstanceHandle,
    pub datareader_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DeleteDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteDataReader) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let Some(data_reader) = subscriber.remove_data_reader(message.datareader_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message
            .participant_address
            .send_actor_mail(discovery_service::AnnounceDeletedDataReader { data_reader })
            .ok();
        message.reply_sender.send(Ok(()));
    }
}

pub struct LookupDataReader {
    pub subscriber_handle: InstanceHandle,
    pub topic_name: String,
    pub reply_sender:
        OneshotSender<DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>)>>>,
}
impl MailHandler<LookupDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: LookupDataReader) {
        if self
            .domain_participant
            .get_topic(&message.topic_name)
            .is_none()
        {
            message.reply_sender.send(Err(DdsError::BadParameter));
            return;
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if self.domain_participant.instance_handle() == message.subscriber_handle {
            message.reply_sender.send(Ok(self
                .domain_participant
                .builtin_subscriber_mut()
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == message.topic_name)
                .map(|x: &mut DataReaderEntity| {
                    (x.instance_handle(), x.status_condition().address())
                })))
        } else {
            let Some(s) = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
            else {
                message.reply_sender.send(Err(DdsError::AlreadyDeleted));
                return;
            };
            message.reply_sender.send(Ok(s
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == message.topic_name)
                .map(|x| (x.instance_handle(), x.status_condition().address()))))
        }
    }
}

pub struct SetDefaultDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub qos: QosKind<DataReaderQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDefaultDataReaderQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDefaultDataReaderQos) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        let qos = match message.qos {
            QosKind::Default => DataReaderQos::default(),
            QosKind::Specific(q) => q,
        };
        message
            .reply_sender
            .send(subscriber.set_default_data_reader_qos(qos));
    }
}

pub struct GetDefaultDataReaderQos {
    pub subscriber_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<DataReaderQos>>,
}
impl MailHandler<GetDefaultDataReaderQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDefaultDataReaderQos) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message
            .reply_sender
            .send(Ok(subscriber.default_data_reader_qos().clone()));
    }
}

pub struct SetQos {
    pub subscriber_handle: InstanceHandle,
    pub qos: QosKind<SubscriberQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) {
        let qos = match message.qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(subscriber.set_qos(qos));
    }
}

pub struct GetSubscriberQos {
    pub subscriber_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<SubscriberQos>>,
}
impl MailHandler<GetSubscriberQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetSubscriberQos) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(subscriber.qos().clone()));
    }
}

pub struct SetListener {
    pub subscriber_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) {
        let listener = message.a_listener.map(|l| {
            Actor::spawn(
                SubscriberListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        subscriber.set_listener(listener, message.mask);
        message.reply_sender.send(Ok(()))
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
