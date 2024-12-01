use crate::{
    dds_async::subscriber_listener::SubscriberListenerAsync,
    implementation::{
        any_data_reader_listener::AnyDataReaderListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::data_reader::DataReaderEntity,
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
        status::StatusKind,
    },
    runtime::actor::{Actor, ActorAddress, Mail, MailHandler},
    transport::{
        history_cache::{CacheChange, HistoryCache},
        types::{
            EntityId, Guid, TopicKind, USER_DEFINED_READER_NO_KEY, USER_DEFINED_READER_WITH_KEY,
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
}
impl Mail for CreateDataReader {
    type Result = DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>;
}
impl MailHandler<CreateDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: CreateDataReader) -> <CreateDataReader as Mail>::Result {
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

        let topic = self
            .domain_participant
            .get_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic_kind = get_topic_kind(topic.type_support().as_ref());
        let topic_name = topic.topic_name().to_owned();
        let type_name = topic.type_name().to_owned();
        let reader_handle = self.instance_handle_counter.generate_new_instance_handle();

        let type_support = topic.type_support().clone();
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => subscriber.default_data_reader_qos().clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        self.entity_counter += 1;
        let prefix = self.transport.guid().prefix();

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
        let reader_guid = Guid::new(prefix, entity_id);
        let transport_reader = self.transport.create_reader(
            reader_guid,
            &message.topic_name,
            topic_kind,
            Box::new(UserDefinedReaderHistoryCache {
                domain_participant_address: message.domain_participant_address.clone(),
                subscriber_handle: subscriber.instance_handle(),
                data_reader_handle: reader_handle,
            }),
        );

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
                })?;
        }
        Ok((data_reader_handle, reader_status_condition_address))
    }
}

pub struct DeleteDataReader {
    pub subscriber_handle: InstanceHandle,
    pub datareader_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for DeleteDataReader {
    type Result = DdsResult<()>;
}
impl MailHandler<DeleteDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteDataReader) -> <DeleteDataReader as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .remove_data_reader(message.datareader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        message
            .participant_address
            .send_actor_mail(discovery_service::AnnounceDeletedDataReader { data_reader })
            .ok();
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
        if self
            .domain_participant
            .get_topic(&message.topic_name)
            .is_none()
        {
            return Err(DdsError::BadParameter);
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if self.domain_participant.instance_handle() == message.subscriber_handle {
            Ok(self
                .domain_participant
                .builtin_subscriber_mut()
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == message.topic_name)
                .map(|x: &mut DataReaderEntity| {
                    (x.instance_handle(), x.status_condition().address())
                }))
        } else {
            let s = self
                .domain_participant
                .get_mut_subscriber(message.subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?;
            Ok(s.data_reader_list_mut()
                .find(|dr| dr.topic_name() == message.topic_name)
                .map(|x| (x.instance_handle(), x.status_condition().address())))
        }
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
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let qos = match message.qos {
            QosKind::Default => DataReaderQos::default(),
            QosKind::Specific(q) => q,
        };
        subscriber.set_default_data_reader_qos(qos)
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
        Ok(self
            .domain_participant
            .get_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .default_data_reader_qos()
            .clone())
    }
}

pub struct SetQos {
    pub subscriber_handle: InstanceHandle,
    pub qos: QosKind<SubscriberQos>,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

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
            .domain_participant
            .get_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .qos()
            .clone())
    }
}

pub struct SetListener {
    pub subscriber_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        let listener = message.a_listener.map(|l| {
            Actor::spawn(
                SubscriberListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        self.domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .set_listener(listener, message.mask);
        Ok(())
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
