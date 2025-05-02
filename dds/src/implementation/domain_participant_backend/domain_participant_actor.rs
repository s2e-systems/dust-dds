use core::{future::Future, pin::Pin};
use std::sync::Arc;

use fnmatch_regex::glob_to_regex;

use super::{
    domain_participant_actor_mail::{DomainParticipantMail, EventServiceMail, MessageServiceMail},
    entities::{
        data_reader::{AddChangeResult, DataReaderEntity},
        data_writer::{DataWriterEntity, TransportWriterKind},
        domain_participant::DomainParticipantEntity,
        publisher::PublisherEntity,
        subscriber::SubscriberEntity,
        topic::TopicEntity,
    },
    handle::InstanceHandleCounter,
};
use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
        domain_participant::DomainParticipantAsync,
        domain_participant_listener::DomainParticipantListenerAsync, publisher::PublisherAsync,
        publisher_listener::PublisherListenerAsync, subscriber::SubscriberAsync,
        subscriber_listener::SubscriberListenerAsync, topic::TopicAsync,
        topic_listener::TopicListenerAsync,
    },
    implementation::{
        any_data_reader_listener::AnyDataReaderListener,
        any_data_writer_listener::AnyDataWriterListener,
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
            spdp_discovered_participant_data::{
                BuiltinEndpointQos, BuiltinEndpointSet, ParticipantProxy,
                SpdpDiscoveredParticipantData,
            },
        },
        domain_participant_backend::entities::data_reader::TransportReaderKind,
        domain_participant_factory::domain_participant_factory_actor::DdsTransportParticipant,
        listeners::{
            data_reader_listener::{self, DataReaderListenerActor},
            data_writer_listener::{self, DataWriterListenerActor},
            domain_participant_listener::{self, DomainParticipantListenerActor},
            publisher_listener::{self, PublisherListenerActor},
            subscriber_listener::{self, SubscriberListenerActor},
            topic_listener::TopicListenerActor,
        },
        status_condition::status_condition_actor::{self, StatusConditionActor},
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_serialized_key_from_serialized_foo,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, QosKind,
            SubscriberQos, TopicQos,
        },
        qos_policy::{
            DurabilityQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind,
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, PublicationMatchedStatus,
            StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind, Time},
    },
    runtime::{
        actor::{Actor, ActorAddress},
        executor::Executor,
        oneshot::oneshot,
        timer::TimerDriver,
    },
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    topic_definition::type_support::{DdsDeserialize, DdsSerialize},
    transport::{
        self,
        history_cache::{CacheChange, HistoryCache},
        types::{
            ChangeKind, DurabilityKind, EntityId, ReliabilityKind, TopicKind, ENTITYID_UNKNOWN,
            USER_DEFINED_READER_NO_KEY, USER_DEFINED_READER_WITH_KEY, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
    },
    xtypes::dynamic_type::DynamicType,
};

pub const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
];

pub struct DomainParticipantActor {
    pub transport: DdsTransportParticipant,
    pub instance_handle_counter: InstanceHandleCounter,
    pub entity_counter: u16,
    pub domain_participant: DomainParticipantEntity,
    pub backend_executor: Executor,
    pub listener_executor: Executor,
    pub timer_driver: TimerDriver,
}

impl DomainParticipantActor {
    pub fn new(
        domain_participant: DomainParticipantEntity,
        transport: DdsTransportParticipant,
        backend_executor: Executor,
        listener_executor: Executor,
        timer_driver: TimerDriver,
        instance_handle_counter: InstanceHandleCounter,
    ) -> Self {
        Self {
            transport,
            instance_handle_counter,
            entity_counter: 0,
            domain_participant,
            backend_executor,
            listener_executor,
            timer_driver,
        }
    }

    pub fn get_participant_async(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> DomainParticipantAsync {
        DomainParticipantAsync::new(
            participant_address,
            self.domain_participant.status_condition().address(),
            self.domain_participant
                .builtin_subscriber()
                .status_condition()
                .address(),
            self.domain_participant.domain_id(),
            self.domain_participant.instance_handle(),
            self.timer_driver.handle(),
        )
    }

    pub fn get_subscriber_async(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<SubscriberAsync> {
        Ok(SubscriberAsync::new(
            subscriber_handle,
            self.domain_participant
                .get_subscriber(subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .status_condition()
                .address(),
            self.get_participant_async(participant_address),
        ))
    }

    pub fn get_data_reader_async<Foo>(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<DataReaderAsync<Foo>> {
        let data_reader = self
            .domain_participant
            .get_subscriber(subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_reader(data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(DataReaderAsync::new(
            data_reader_handle,
            data_reader.status_condition().address(),
            self.get_subscriber_async(participant_address.clone(), subscriber_handle)?,
            self.get_topic_async(participant_address, data_reader.topic_name().to_owned())?,
        ))
    }

    pub fn get_publisher_async(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<PublisherAsync> {
        Ok(PublisherAsync::new(
            publisher_handle,
            self.domain_participant
                .get_publisher(publisher_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .status_condition()
                .address(),
            self.get_participant_async(participant_address),
        ))
    }

    pub fn get_data_writer_async<Foo>(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<DataWriterAsync<Foo>> {
        let data_writer = self
            .domain_participant
            .get_publisher(publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(DataWriterAsync::new(
            data_writer_handle,
            data_writer.status_condition().address(),
            self.get_publisher_async(participant_address.clone(), publisher_handle)?,
            self.get_topic_async(participant_address, data_writer.topic_name().to_owned())?,
        ))
    }

    pub fn get_topic_async(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_name: String,
    ) -> DdsResult<TopicAsync> {
        let topic = self
            .domain_participant
            .get_topic(&topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;
        Ok(TopicAsync::new(
            topic.instance_handle(),
            topic.status_condition().address(),
            topic.type_name().to_owned(),
            topic_name,
            self.get_participant_async(participant_address),
        ))
    }

    pub fn get_inconsistent_topic_status(
        &mut self,
        topic_name: String,
    ) -> DdsResult<InconsistentTopicStatus> {
        let Some(topic) = self.domain_participant.get_mut_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(topic.get_inconsistent_topic_status())
    }

    pub fn set_topic_qos(
        &mut self,
        topic_name: String,
        topic_qos: QosKind<TopicQos>,
    ) -> DdsResult<()> {
        let qos = match topic_qos {
            QosKind::Default => self.domain_participant.get_default_topic_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(topic) = self.domain_participant.get_mut_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };

        topic.set_qos(qos)
    }

    pub fn get_topic_qos(&mut self, topic_name: String) -> DdsResult<TopicQos> {
        let Some(topic) = self.domain_participant.get_mut_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(topic.qos().clone())
    }

    pub fn enable_topic(&mut self, topic_name: String) -> DdsResult<()> {
        let Some(topic) = self.domain_participant.get_mut_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !topic.enabled() {
            topic.enable();
            self.announce_topic(topic_name);
        }

        Ok(())
    }

    pub fn get_type_support(
        &mut self,
        topic_name: String,
    ) -> DdsResult<Arc<dyn DynamicType + Send + Sync>> {
        let Some(topic) = self.domain_participant.get_mut_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(topic.type_support().clone())
    }

    pub fn create_user_defined_publisher(
        &mut self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)> {
        let publisher_qos = match qos {
            QosKind::Default => self.domain_participant.default_publisher_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let publisher_handle = self.instance_handle_counter.generate_new_instance_handle();
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let publisher_status_condition_address = status_condition.address();
        let listener = a_listener.map(|l| {
            Actor::spawn(
                PublisherListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let mut publisher = PublisherEntity::new(
            publisher_qos,
            publisher_handle,
            listener,
            mask,
            status_condition,
        );

        if self.domain_participant.enabled()
            && self
                .domain_participant
                .qos()
                .entity_factory
                .autoenable_created_entities
        {
            publisher.enable();
        }

        self.domain_participant.insert_publisher(publisher);

        Ok((publisher_handle, publisher_status_condition_address))
    }

    pub fn delete_user_defined_publisher(
        &mut self,
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<()> {
        if participant_handle != self.domain_participant.instance_handle() {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }
        let Some(publisher) = self.domain_participant.get_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        if publisher.data_writer_list().count() > 0 {
            return Err(DdsError::PreconditionNotMet(
                "Publisher still contains data writers".to_string(),
            ));
        }
        let Some(_) = self.domain_participant.remove_publisher(&publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(())
    }

    pub fn create_user_defined_subscriber(
        &mut self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)> {
        let subscriber_qos = match qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let subscriber_handle = self.instance_handle_counter.generate_new_instance_handle();
        let listener = a_listener.map(|l| {
            Actor::spawn(
                SubscriberListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let listener_mask = mask.to_vec();

        let mut subscriber = SubscriberEntity::new(
            subscriber_handle,
            subscriber_qos,
            Actor::spawn(
                StatusConditionActor::default(),
                &self.listener_executor.handle(),
            ),
            listener,
            listener_mask,
        );

        let subscriber_status_condition_address = subscriber.status_condition().address();

        if self.domain_participant.enabled()
            && self
                .domain_participant
                .qos()
                .entity_factory
                .autoenable_created_entities
        {
            subscriber.enable();
        }

        self.domain_participant.insert_subscriber(subscriber);

        Ok((subscriber_handle, subscriber_status_condition_address))
    }

    pub fn delete_user_defined_subscriber(
        &mut self,
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<()> {
        if self.domain_participant.instance_handle() != participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        let Some(subscriber) = self.domain_participant.get_subscriber(subscriber_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        if subscriber.data_reader_list().count() > 0 {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber still contains data readers".to_string(),
            ));
        }
        let Some(_) = self
            .domain_participant
            .remove_subscriber(&subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(())
    }

    pub fn create_topic(
        &mut self,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn TopicListenerAsync + Send>>,
        mask: Vec<StatusKind>,
        type_support: Arc<dyn DynamicType + Send + Sync>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)> {
        if self.domain_participant.get_topic(&topic_name).is_some() {
            return Err(DdsError::PreconditionNotMet(format!(
                "Topic with name {} already exists.
         To access this topic call the lookup_topicdescription method.",
                topic_name
            )));
        }

        let qos = match qos {
            QosKind::Default => self.domain_participant.get_default_topic_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let topic_handle = self.instance_handle_counter.generate_new_instance_handle();
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let topic_status_condition_address = status_condition.address();
        let topic_listener = a_listener
            .map(|l| Actor::spawn(TopicListenerActor::new(l), &self.listener_executor.handle()));
        let topic = TopicEntity::new(
            qos,
            type_name,
            topic_name.clone(),
            topic_handle,
            status_condition,
            topic_listener,
            mask,
            type_support,
        );

        self.domain_participant.insert_topic(topic);

        if self.domain_participant.enabled()
            && self
                .domain_participant
                .qos()
                .entity_factory
                .autoenable_created_entities
        {
            self.enable_topic(topic_name)?;
        }

        Ok((topic_handle, topic_status_condition_address))
    }

    pub fn delete_user_defined_topic(
        &mut self,
        participant_handle: InstanceHandle,
        topic_name: String,
    ) -> DdsResult<()> {
        if self.domain_participant.instance_handle() != participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ));
        }

        if BUILT_IN_TOPIC_NAME_LIST.contains(&topic_name.as_str()) {
            return Ok(());
        }

        let Some(topic) = self.domain_participant.get_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };

        if Arc::strong_count(topic.type_support()) > 1 {
            return Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data writer or data reader".to_string(),
            ));
        }

        let Some(_) = self.domain_participant.remove_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(())
    }

    pub fn find_topic(
        &mut self,
        topic_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
    ) -> DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>, String)>> {
        if let Some(topic) = self.domain_participant.get_topic(&topic_name) {
            Ok(Some((
                topic.instance_handle(),
                topic.status_condition().address(),
                topic.type_name().to_owned(),
            )))
        } else {
            if let Some(discovered_topic_data) = self.domain_participant.find_topic(&topic_name) {
                let qos = TopicQos {
                    topic_data: discovered_topic_data.topic_data().clone(),
                    durability: discovered_topic_data.durability().clone(),
                    deadline: discovered_topic_data.deadline().clone(),
                    latency_budget: discovered_topic_data.latency_budget().clone(),
                    liveliness: discovered_topic_data.liveliness().clone(),
                    reliability: discovered_topic_data.reliability().clone(),
                    destination_order: discovered_topic_data.destination_order().clone(),
                    history: discovered_topic_data.history().clone(),
                    resource_limits: discovered_topic_data.resource_limits().clone(),
                    transport_priority: discovered_topic_data.transport_priority().clone(),
                    lifespan: discovered_topic_data.lifespan().clone(),
                    ownership: discovered_topic_data.ownership().clone(),
                    representation: discovered_topic_data.representation().clone(),
                };
                let type_name = discovered_topic_data.type_name.clone();
                let topic_handle = self.instance_handle_counter.generate_new_instance_handle();
                let mut topic = TopicEntity::new(
                    qos,
                    type_name.clone(),
                    topic_name.clone(),
                    topic_handle,
                    Actor::spawn(
                        StatusConditionActor::default(),
                        &self.listener_executor.handle(),
                    ),
                    None,
                    vec![],
                    type_support,
                );
                topic.enable();
                let topic_status_condition_address = topic.status_condition().address();

                self.domain_participant.insert_topic(topic);
                return Ok(Some((
                    topic_handle,
                    topic_status_condition_address,
                    type_name,
                )));
            }
            Ok(None)
        }
    }

    pub fn lookup_topicdescription(
        &mut self,
        topic_name: String,
    ) -> DdsResult<Option<(String, InstanceHandle, ActorAddress<StatusConditionActor>)>> {
        if let Some(topic) = self.domain_participant.get_topic(&topic_name) {
            Ok(Some((
                topic.type_name().to_owned(),
                topic.instance_handle(),
                topic.status_condition().address(),
            )))
        } else {
            Ok(None)
        }
    }

    pub fn ignore_participant(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_participant(handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    pub fn ignore_subscription(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_subscription(handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }
    pub fn ignore_publication(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.domain_participant.enabled() {
            self.domain_participant.ignore_publication(handle);
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    pub fn delete_participant_contained_entities(&mut self) -> DdsResult<()> {
        let deleted_publisher_list: Vec<PublisherEntity> =
            self.domain_participant.drain_publisher_list().collect();
        for mut publisher in deleted_publisher_list {
            for data_writer in publisher.drain_data_writer_list() {
                self.announce_deleted_data_writer(data_writer);
            }
        }

        let deleted_subscriber_list: Vec<SubscriberEntity> =
            self.domain_participant.drain_subscriber_list().collect();
        for mut subscriber in deleted_subscriber_list {
            for data_reader in subscriber.drain_data_reader_list() {
                self.announce_deleted_data_reader(data_reader);
            }
        }

        self.domain_participant.delete_all_topics();

        Ok(())
    }

    pub fn set_default_publisher_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_default_publisher_qos(qos);
        Ok(())
    }

    pub fn get_default_publisher_qos(&mut self) -> DdsResult<PublisherQos> {
        Ok(self.domain_participant.default_publisher_qos().clone())
    }

    pub fn set_default_subscriber_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_default_subscriber_qos(qos);

        Ok(())
    }

    pub fn get_default_subscriber_qos(&mut self) -> DdsResult<SubscriberQos> {
        Ok(self.domain_participant.default_subscriber_qos().clone())
    }

    pub fn set_default_topic_qos(&mut self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => TopicQos::default(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };

        self.domain_participant.set_default_topic_qos(qos)
    }

    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.domain_participant.get_default_topic_qos().clone())
    }

    pub fn get_discovered_participants(&mut self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self.domain_participant.get_discovered_participants())
    }

    pub fn get_discovered_participant_data(
        &mut self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        let Some(handle) = self
            .domain_participant
            .get_discovered_participant_data(&participant_handle)
        else {
            return Err(DdsError::BadParameter);
        };
        Ok(handle.dds_participant_data.clone())
    }

    pub fn get_discovered_topics(&mut self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self.domain_participant.get_discovered_topics())
    }

    pub fn get_discovered_topic_data(
        &mut self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        let Some(handle) = self
            .domain_participant
            .get_discovered_topic_data(&topic_handle)
        else {
            return Err(DdsError::PreconditionNotMet(
                "Topic with this handle not discovered".to_owned(),
            ));
        };

        Ok(handle.clone())
    }

    pub fn get_current_time(&mut self) -> Time {
        self.domain_participant.get_current_time()
    }

    pub fn set_domain_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.set_qos(qos);
        if self.domain_participant.enabled() {
            self.announce_participant();
        }
        Ok(())
    }

    pub fn get_domain_participant_qos(&mut self) -> DdsResult<DomainParticipantQos> {
        Ok(self.domain_participant.qos().clone())
    }

    pub fn set_domain_participant_listener(
        &mut self,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let participant_listener = listener.map(|l| {
            Actor::spawn(
                DomainParticipantListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        self.domain_participant
            .set_listener(participant_listener, status_kind);
        Ok(())
    }

    pub fn enable_domain_participant(&mut self) -> DdsResult<()> {
        if !self.domain_participant.enabled() {
            self.domain_participant.enable();

            self.announce_participant();
        }

        Ok(())
    }

    pub fn is_participant_empty(&mut self) -> bool {
        self.domain_participant.is_empty()
    }

    pub fn create_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener>>,
        mask: Vec<StatusKind>,
        domain_participant_address: ActorAddress<DomainParticipantActor>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)> {
        struct UserDefinedReaderHistoryCache {
            pub domain_participant_address: ActorAddress<DomainParticipantActor>,
            pub subscriber_handle: InstanceHandle,
            pub data_reader_handle: InstanceHandle,
        }

        impl HistoryCache for UserDefinedReaderHistoryCache {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.domain_participant_address
                    .send_actor_mail(DomainParticipantMail::Message(
                        MessageServiceMail::AddCacheChange {
                            participant_address: self.domain_participant_address.clone(),
                            cache_change,
                            subscriber_handle: self.subscriber_handle,
                            data_reader_handle: self.data_reader_handle,
                        },
                    ))
                    .ok();
            }

            fn remove_change(&mut self, _sequence_number: i64) {
                todo!()
            }
        }

        let Some(topic) = self.domain_participant.get_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic_kind = get_topic_kind(topic.type_support().as_ref());
        let topic_name = topic.topic_name().to_owned();
        let type_name = topic.type_name().to_owned();
        let reader_handle = self.instance_handle_counter.generate_new_instance_handle();

        let type_support = topic.type_support().clone();
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos().clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
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
                    domain_participant_address: domain_participant_address.clone(),
                    subscriber_handle: subscriber.instance_handle(),
                    data_reader_handle: reader_handle,
                }),
            ));

        let listener_mask = mask.to_vec();
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let listener = a_listener.map(|l| {
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
            self.enable_data_reader(
                subscriber_handle,
                data_reader_handle,
                domain_participant_address,
            )?;
        }
        Ok((data_reader_handle, reader_status_condition_address))
    }

    pub fn delete_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        datareader_handle: InstanceHandle,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.remove_data_reader(datareader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        self.announce_deleted_data_reader(data_reader);
        Ok(())
    }

    pub fn lookup_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        topic_name: String,
    ) -> DdsResult<Option<(InstanceHandle, ActorAddress<StatusConditionActor>)>> {
        if self.domain_participant.get_topic(&topic_name).is_none() {
            return Err(DdsError::BadParameter);
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if self.domain_participant.instance_handle() == subscriber_handle {
            Ok(self
                .domain_participant
                .builtin_subscriber_mut()
                .data_reader_list_mut()
                .find(|dr| dr.topic_name() == topic_name)
                .map(|x: &mut DataReaderEntity| {
                    (x.instance_handle(), x.status_condition().address())
                }))
        } else {
            let Some(s) = self
                .domain_participant
                .get_mut_subscriber(subscriber_handle)
            else {
                return Err(DdsError::AlreadyDeleted);
            };
            Ok(s.data_reader_list_mut()
                .find(|dr| dr.topic_name() == topic_name)
                .map(|x| (x.instance_handle(), x.status_condition().address())))
        }
    }

    pub fn set_default_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => DataReaderQos::default(),
            QosKind::Specific(q) => q,
        };
        subscriber.set_default_data_reader_qos(qos)
    }

    pub fn get_default_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<DataReaderQos> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(subscriber.default_data_reader_qos().clone())
    }

    pub fn set_subscriber_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos().clone(),
            QosKind::Specific(q) => q,
        };

        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        subscriber.set_qos(qos)
    }

    pub fn get_subscriber_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<SubscriberQos> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(subscriber.qos().clone())
    }

    pub fn set_subscriber_listener(
        &mut self,
        subscriber_handle: InstanceHandle,
        a_listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let listener = a_listener.map(|l| {
            Actor::spawn(
                SubscriberListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        subscriber.set_listener(listener, mask);
        Ok(())
    }

    pub fn create_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        mask: Vec<StatusKind>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)> {
        let Some(topic) = self.domain_participant.get_topic(&topic_name) else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic_kind = get_topic_kind(topic.type_support().as_ref());
        let type_support = topic.type_support().clone();
        let type_name = topic.type_name().to_owned();
        let entity_kind = match topic_kind {
            TopicKind::WithKey => USER_DEFINED_WRITER_WITH_KEY,
            TopicKind::NoKey => USER_DEFINED_WRITER_NO_KEY,
        };

        self.entity_counter += 1;
        let entity_id = EntityId::new(
            [
                0,
                self.entity_counter.to_le_bytes()[0],
                self.entity_counter.to_le_bytes()[1],
            ],
            entity_kind,
        );

        let writer_handle = self.instance_handle_counter.generate_new_instance_handle();
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => publisher.default_datawriter_qos().clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    return Err(DdsError::InconsistentPolicy);
                }
            }
        };
        let reliablity_kind = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        };
        let transport_writer = self
            .transport
            .create_stateful_writer(entity_id, reliablity_kind);

        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let writer_status_condition_address = status_condition.address();
        let listener = a_listener.map(|l| {
            Actor::spawn(
                DataWriterListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let data_writer = DataWriterEntity::new(
            writer_handle,
            TransportWriterKind::Stateful(transport_writer),
            topic_name,
            type_name,
            type_support,
            status_condition,
            listener,
            mask,
            qos,
        );
        let data_writer_handle = data_writer.instance_handle();

        publisher.insert_data_writer(data_writer);

        if publisher.enabled() && publisher.qos().entity_factory.autoenable_created_entities {
            self.enable_data_writer(publisher_handle, writer_handle, participant_address)?;
        }

        Ok((data_writer_handle, writer_status_condition_address))
    }

    pub fn delete_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        datawriter_handle: InstanceHandle,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_writer) = publisher.remove_data_writer(datawriter_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        self.announce_deleted_data_writer(data_writer);
        Ok(())
    }

    pub fn get_default_datawriter_qos(
        &mut self,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(publisher.default_datawriter_qos().clone())
    }

    pub fn set_default_datawriter_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => q,
        };
        publisher.set_default_datawriter_qos(qos)
    }

    pub fn set_publisher_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_publisher_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        publisher.set_qos(qos)
    }

    pub fn get_publisher_qos(
        &mut self,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<PublisherQos> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(publisher.qos().clone())
    }

    pub fn set_publisher_listener(
        &mut self,
        publisher_handle: InstanceHandle,
        a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        publisher.set_listener(
            a_listener.map(|l| {
                Actor::spawn(
                    PublisherListenerActor::new(l),
                    &self.listener_executor.handle(),
                )
            }),
            mask,
        );
        Ok(())
    }

    pub fn get_publication_matched_status(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<PublicationMatchedStatus> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let status = data_writer.get_publication_matched_status();

        data_writer.status_condition().send_actor_mail(
            status_condition_actor::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            },
        );
        Ok(status)
    }

    pub fn set_listener_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        listener_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let listener = listener.map(|l| {
            Actor::spawn(
                DataWriterListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Ok(());
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return Ok(());
        };

        data_writer.set_listener(listener, listener_mask);

        Ok(())
    }

    pub fn get_data_writer_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.qos().clone())
    }

    pub fn get_matched_subscriptions(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_writer.get_matched_subscriptions())
    }

    pub fn get_matched_subscription_data(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_writer
            .get_matched_subscription_data(&subscription_handle)
            .ok_or(DdsError::BadParameter)
            .cloned()
    }

    pub fn unregister_instance(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let serialized_key = match get_serialized_key_from_serialized_foo(
            &serialized_data,
            data_writer.type_support(),
        ) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };
        data_writer.unregister_w_timestamp(serialized_key, timestamp)
    }

    pub fn lookup_instance(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
    ) -> DdsResult<Option<InstanceHandle>> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !data_writer.enabled() {
            return Err(DdsError::NotEnabled);
        }

        let instance_handle = match get_instance_handle_from_serialized_foo(
            &serialized_data,
            data_writer.type_support(),
        ) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(data_writer
            .contains_instance(&instance_handle)
            .then_some(instance_handle))
    }

    pub fn write_w_timestamp(
        &mut self,
        participant_address: ActorAddress<DomainParticipantActor>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let now = self.domain_participant.get_current_time();
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let instance_handle = match get_instance_handle_from_serialized_foo(
            &serialized_data,
            data_writer.type_support(),
        ) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };

        match data_writer.qos().lifespan.duration {
            DurationKind::Finite(lifespan_duration) => {
                let timer_handle = self.timer_driver.handle();
                let sleep_duration = timestamp - now + lifespan_duration;
                if sleep_duration > Duration::new(0, 0) {
                    let sequence_number =
                        match data_writer.write_w_timestamp(serialized_data, timestamp) {
                            Ok(s) => s,
                            Err(e) => {
                                return Err(e);
                            }
                        };

                    let participant_address = participant_address.clone();
                    self.backend_executor.handle().spawn(async move {
                        timer_handle.sleep(sleep_duration.into()).await;
                        participant_address
                            .send_actor_mail(DomainParticipantMail::Message(
                                MessageServiceMail::RemoveWriterChange {
                                    publisher_handle,
                                    data_writer_handle,
                                    sequence_number,
                                },
                            ))
                            .ok();
                    });
                }
            }
            DurationKind::Infinite => {
                match data_writer.write_w_timestamp(serialized_data, timestamp) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(e);
                    }
                };
            }
        }

        if let DurationKind::Finite(deadline_missed_period) = data_writer.qos().deadline.period {
            let timer_handle = self.timer_driver.handle();
            let offered_deadline_missed_task = self.backend_executor.handle().spawn(async move {
                loop {
                    timer_handle.sleep(deadline_missed_period.into()).await;
                    participant_address
                        .send_actor_mail(DomainParticipantMail::Event(
                            EventServiceMail::OfferedDeadlineMissed {
                                publisher_handle,
                                data_writer_handle,
                                change_instance_handle: instance_handle,
                                participant_address: participant_address.clone(),
                            },
                        ))
                        .ok();
                }
            });
            data_writer.insert_instance_deadline_missed_task(
                instance_handle,
                offered_deadline_missed_task,
            );
        }

        Ok(())
    }

    pub fn dispose_w_timestamp(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let serialized_key = match get_serialized_key_from_serialized_foo(
            &serialized_data,
            data_writer.type_support(),
        ) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };
        data_writer.dispose_w_timestamp(serialized_key, timestamp)
    }

    pub fn wait_for_acknowledgments(
        &mut self,
        participant_address: ActorAddress<DomainParticipantActor>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = DdsResult<()>> + Send>> {
        let timer_handle = self.timer_driver.handle();
        Box::pin(async move {
            timer_handle
                .timeout(
                    timeout.into(),
                    Box::pin(async move {
                        loop {
                            let (reply_sender, reply_receiver) = oneshot();
                            participant_address
                                .send_actor_mail(DomainParticipantMail::Message(
                                    MessageServiceMail::AreAllChangesAcknowledged {
                                        publisher_handle,
                                        data_writer_handle,
                                        reply_sender,
                                    },
                                ))
                                .ok();
                            let reply = reply_receiver.await;
                            match reply {
                                Ok(are_changes_acknowledged) => match are_changes_acknowledged {
                                    Ok(true) => return Ok(()),
                                    Ok(false) => (),
                                    Err(e) => return Err(e),
                                },
                                Err(e) => {
                                    return Err(DdsError::Error(format!("Channel error: {:?}", e)))
                                }
                            }
                        }
                    }),
                )
                .await
                .map_err(|_| DdsError::Timeout)?
        })
    }

    pub fn get_offered_deadline_missed_status(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.get_offered_deadline_missed_status())
    }

    pub fn enable_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_writer.enabled() {
            data_writer.enable();

            let discovered_reader_list: Vec<_> = self
                .domain_participant
                .discovered_reader_data_list()
                .cloned()
                .collect();
            for discovered_reader_data in discovered_reader_list {
                self.add_discovered_reader(
                    discovered_reader_data,
                    publisher_handle,
                    data_writer_handle,
                    participant_address.clone(),
                );
            }

            self.announce_data_writer(publisher_handle, data_writer_handle);
        }
        Ok(())
    }

    pub fn set_data_writer_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        qos: QosKind<DataWriterQos>,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => publisher.default_datawriter_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_writer) = publisher
            .data_writer_list_mut()
            .find(|x| x.instance_handle() == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        match data_writer.set_qos(qos) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        }
        if data_writer.enabled() {
            self.announce_data_writer(publisher_handle, data_writer_handle);
        }
        Ok(())
    }

    pub fn are_all_changes_acknowledged(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        let Some(publisher) = self.domain_participant.get_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_writer) = publisher.get_data_writer(data_writer_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_writer.are_all_changes_acknowledged())
    }

    pub fn read(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>> {
        let subscriber = if subscriber_handle == self.domain_participant.instance_handle() {
            Some(self.domain_participant.builtin_subscriber_mut())
        } else {
            self.domain_participant
                .get_mut_subscriber(subscriber_handle)
        };

        let Some(subscriber) = subscriber else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_reader.read(
            max_samples,
            &sample_states,
            &view_states,
            &instance_states,
            specific_instance_handle,
        )
    }

    pub fn take(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader.take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn read_next_instance(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader.read_next_instance(
            max_samples,
            previous_handle,
            &sample_states,
            &view_states,
            &instance_states,
        )
    }

    pub fn take_next_instance(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader.take_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn get_subscription_matched_status(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionMatchedStatus> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let status = data_reader.get_subscription_matched_status();
        data_reader.status_condition().send_actor_mail(
            status_condition_actor::RemoveCommunicationState {
                state: StatusKind::SubscriptionMatched,
            },
        );
        Ok(status)
    }

    pub fn wait_for_historical_data(
        &mut self,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_wait: Duration,
    ) -> Pin<Box<dyn Future<Output = DdsResult<()>> + Send>> {
        let timer_handle = self.timer_driver.handle();

        Box::pin(async move {
            timer_handle
                .timeout(
                    max_wait.into(),
                    Box::pin(async move {
                        loop {
                            let (reply_sender, reply_receiver) = oneshot();
                            participant_address.send_actor_mail(DomainParticipantMail::Message(
                                MessageServiceMail::IsHistoricalDataReceived {
                                    subscriber_handle: subscriber_handle,
                                    data_reader_handle: data_reader_handle,
                                    reply_sender,
                                },
                            ))?;

                            let reply = reply_receiver.await;
                            match reply {
                                Ok(historical_data_received) => match historical_data_received {
                                    Ok(true) => return Ok(()),
                                    Ok(false) => (),
                                    Err(e) => return Err(e),
                                },
                                Err(e) => {
                                    return Err(DdsError::Error(format!("Channel error: {:?}", e)))
                                }
                            }
                        }
                    }),
                )
                .await
                .map_err(|_| DdsError::Timeout)?
        })
    }

    pub fn get_matched_publication_data(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled() {
            return Err(DdsError::NotEnabled);
        }

        data_reader
            .get_matched_publication_data(&publication_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn get_matched_publications(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_reader.get_matched_publications())
    }

    pub fn set_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        match data_reader.set_qos(qos) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        if data_reader.enabled() {
            self.announce_data_reader(subscriber_handle, data_reader_handle);
        }

        Ok(())
    }

    pub fn get_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<DataReaderQos> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_reader.qos().clone())
    }

    pub fn set_data_reader_listener(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        listener: Option<Box<dyn AnyDataReaderListener>>,
        listener_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let listener = listener.map(|l| {
            Actor::spawn(
                DataReaderListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader.set_listener(listener, listener_mask);
        Ok(())
    }

    pub fn is_historical_data_received(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        let Some(subscriber) = self.domain_participant.get_subscriber(subscriber_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber.get_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled() {
            return Err(DdsError::NotEnabled);
        };

        match data_reader.qos().durability.kind {
            DurabilityQosPolicyKind::Volatile => {
                return Err(DdsError::IllegalOperation);
            }
            DurabilityQosPolicyKind::TransientLocal
            | DurabilityQosPolicyKind::Transient
            | DurabilityQosPolicyKind::Persistent => (),
        };

        if let TransportReaderKind::Stateful(r) = data_reader.transport_reader() {
            Ok(r.is_historical_data_received())
        } else {
            Ok(true)
        }
    }

    pub fn enable_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled() {
            data_reader.enable();

            let discovered_writer_list: Vec<_> = self
                .domain_participant
                .publication_builtin_topic_data_list()
                .cloned()
                .collect();
            for discovered_writer_data in discovered_writer_list {
                self.add_discovered_writer(
                    discovered_writer_data,
                    subscriber_handle,
                    data_reader_handle,
                    participant_address.clone(),
                );
            }

            self.announce_data_reader(subscriber_handle, data_reader_handle);
        }
        Ok(())
    }

    pub fn announce_participant(&mut self) {
        if self.domain_participant.enabled() {
            let participant_builtin_topic_data = ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.transport.guid().into(),
                },
                user_data: self.domain_participant.qos().user_data.clone(),
            };
            let participant_proxy = ParticipantProxy {
                domain_id: Some(self.domain_participant.domain_id()),
                domain_tag: self.domain_participant.domain_tag().to_owned(),
                protocol_version: self.transport.protocol_version(),
                guid_prefix: self.transport.guid().prefix(),
                vendor_id: self.transport.vendor_id(),
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: self
                    .transport
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                metatraffic_multicast_locator_list: self
                    .transport
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                default_unicast_locator_list: self
                    .transport
                    .default_unicast_locator_list()
                    .to_vec(),
                default_multicast_locator_list: self
                    .transport
                    .default_multicast_locator_list()
                    .to_vec(),
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: 0,
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            };
            let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
                dds_participant_data: participant_builtin_topic_data,
                participant_proxy,
                lease_duration: Duration::new(100, 0),
                discovered_participant_list: self.domain_participant.get_discovered_participants(),
            };
            let timestamp = self.domain_participant.get_current_time();

            if let Some(dw) = self
                .domain_participant
                .builtin_publisher_mut()
                .lookup_datawriter_mut(DCPS_PARTICIPANT)
            {
                if let Ok(serialized_data) = spdp_discovered_participant_data.serialize_data() {
                    dw.write_w_timestamp(serialized_data, timestamp).ok();
                }
            }
        }
    }

    pub fn announce_deleted_participant(&mut self) {
        if self.domain_participant.enabled() {
            let timestamp = self.domain_participant.get_current_time();
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher_mut()
                .lookup_datawriter_mut(DCPS_PARTICIPANT)
            {
                let key = InstanceHandle::new(self.transport.guid().into());
                if let Ok(serialized_data) = key.serialize_data() {
                    dw.dispose_w_timestamp(serialized_data, timestamp).ok();
                }
            }
        }
    }

    fn announce_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) {
        let Some(publisher) = self.domain_participant.get_publisher(publisher_handle) else {
            return;
        };
        let Some(data_writer) = publisher.get_data_writer(data_writer_handle) else {
            return;
        };
        let Some(topic) = self.domain_participant.get_topic(data_writer.topic_name()) else {
            return;
        };

        let topic_data = topic.qos().topic_data.clone();

        let dds_publication_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey {
                value: data_writer.transport_writer().guid().into(),
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
            topic_data,
            group_data: publisher.qos().group_data.clone(),
            representation: data_writer.qos().representation.clone(),
        };
        let writer_proxy = WriterProxy {
            remote_writer_guid: data_writer.transport_writer().guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
        };
        let discovered_writer_data = DiscoveredWriterData {
            dds_publication_data,
            writer_proxy,
        };
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_PUBLICATION)
        {
            if let Ok(serialized_data) = discovered_writer_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }

    fn announce_deleted_data_writer(&mut self, data_writer: DataWriterEntity) {
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_PUBLICATION)
        {
            let key = InstanceHandle::new(data_writer.transport_writer().guid().into());
            if let Ok(serialized_data) = key.serialize_data() {
                dw.dispose_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }

    fn announce_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let Some(subscriber) = self.domain_participant.get_subscriber(subscriber_handle) else {
            return;
        };
        let Some(data_reader) = subscriber.get_data_reader(data_reader_handle) else {
            return;
        };
        let Some(topic) = self.domain_participant.get_topic(data_reader.topic_name()) else {
            return;
        };

        let guid = data_reader.transport_reader().guid();
        let dds_subscription_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey { value: guid.into() },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: data_reader.topic_name().to_owned(),
            type_name: data_reader.type_name().to_owned(),
            durability: data_reader.qos().durability.clone(),
            deadline: data_reader.qos().deadline.clone(),
            latency_budget: data_reader.qos().latency_budget.clone(),
            liveliness: data_reader.qos().liveliness.clone(),
            reliability: data_reader.qos().reliability.clone(),
            ownership: data_reader.qos().ownership.clone(),
            destination_order: data_reader.qos().destination_order.clone(),
            user_data: data_reader.qos().user_data.clone(),
            time_based_filter: data_reader.qos().time_based_filter.clone(),
            presentation: subscriber.qos().presentation.clone(),
            partition: subscriber.qos().partition.clone(),
            topic_data: topic.qos().topic_data.clone(),
            group_data: subscriber.qos().group_data.clone(),
            representation: data_reader.qos().representation.clone(),
        };
        let reader_proxy = ReaderProxy {
            remote_reader_guid: data_reader.transport_reader().guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        };
        let discovered_reader_data = DiscoveredReaderData {
            dds_subscription_data,
            reader_proxy,
        };
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_SUBSCRIPTION)
        {
            if let Ok(serialized_data) = discovered_reader_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }

    fn announce_deleted_data_reader(&mut self, data_reader: DataReaderEntity) {
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_SUBSCRIPTION)
        {
            let guid = data_reader.transport_reader().guid();
            let key = InstanceHandle::new(guid.into());
            if let Ok(serialized_data) = key.serialize_data() {
                dw.dispose_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }

    fn announce_topic(&mut self, topic_name: String) {
        let Some(topic) = self.domain_participant.get_topic(&topic_name) else {
            return;
        };

        let topic_builtin_topic_data = TopicBuiltinTopicData {
            key: BuiltInTopicKey {
                value: topic.instance_handle().into(),
            },
            name: topic.topic_name().to_owned(),
            type_name: topic.type_name().to_owned(),
            durability: topic.qos().durability.clone(),
            deadline: topic.qos().deadline.clone(),
            latency_budget: topic.qos().latency_budget.clone(),
            liveliness: topic.qos().liveliness.clone(),
            reliability: topic.qos().reliability.clone(),
            transport_priority: topic.qos().transport_priority.clone(),
            lifespan: topic.qos().lifespan.clone(),
            destination_order: topic.qos().destination_order.clone(),
            history: topic.qos().history.clone(),
            resource_limits: topic.qos().resource_limits.clone(),
            ownership: topic.qos().ownership.clone(),
            topic_data: topic.qos().topic_data.clone(),
            representation: topic.qos().representation.clone(),
        };

        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_TOPIC)
        {
            if let Ok(serialized_data) = topic_builtin_topic_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }

    fn add_discovered_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == discovered_reader_data
                        .reader_proxy
                        .remote_reader_guid
                        .prefix()
            }) {
            p.participant_proxy.default_unicast_locator_list.clone()
        } else {
            vec![]
        };
        let default_multicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == discovered_reader_data
                        .reader_proxy
                        .remote_reader_guid
                        .prefix()
            }) {
            p.participant_proxy.default_multicast_locator_list.clone()
        } else {
            vec![]
        };
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return;
        };

        let is_any_name_matched = discovered_reader_data
            .dds_subscription_data
            .partition
            .name
            .iter()
            .any(|n| publisher.qos().partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_reader_data
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

        let is_partition_matched = discovered_reader_data.dds_subscription_data.partition
            == publisher.qos().partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let publisher_qos = publisher.qos().clone();
            let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
                return;
            };

            let is_matched_topic_name = discovered_reader_data.dds_subscription_data.topic_name()
                == data_writer.topic_name();
            let is_matched_type_name = discovered_reader_data.dds_subscription_data.get_type_name()
                == data_writer.type_name();

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_reader_incompatible_qos_policy_list(
                        data_writer.qos(),
                        &discovered_reader_data.dds_subscription_data,
                        &publisher_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_writer.add_matched_subscription(
                        discovered_reader_data.dds_subscription_data.clone(),
                    );

                    let unicast_locator_list = if discovered_reader_data
                        .reader_proxy
                        .unicast_locator_list
                        .is_empty()
                    {
                        default_unicast_locator_list
                    } else {
                        discovered_reader_data.reader_proxy.unicast_locator_list
                    };
                    let multicast_locator_list = if discovered_reader_data
                        .reader_proxy
                        .multicast_locator_list
                        .is_empty()
                    {
                        default_multicast_locator_list
                    } else {
                        discovered_reader_data.reader_proxy.multicast_locator_list
                    };
                    let reliability_kind = match discovered_reader_data
                        .dds_subscription_data
                        .reliability
                        .kind
                    {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind =
                        match discovered_reader_data.dds_subscription_data.durability.kind {
                            DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                            DurabilityQosPolicyKind::TransientLocal => {
                                DurabilityKind::TransientLocal
                            }
                            DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                            DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                        };

                    let reader_proxy = transport::writer::ReaderProxy {
                        remote_reader_guid: discovered_reader_data.reader_proxy.remote_reader_guid,
                        remote_group_entity_id: discovered_reader_data
                            .reader_proxy
                            .remote_group_entity_id,
                        reliability_kind,
                        durability_kind,
                        unicast_locator_list,
                        multicast_locator_list,
                        expects_inline_qos: false,
                    };
                    if let TransportWriterKind::Stateful(w) = data_writer.transport_writer_mut() {
                        w.add_matched_reader(reader_proxy);
                    }

                    if data_writer
                        .listener_mask()
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let status = data_writer.get_publication_matched_status();
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) =
                            self.domain_participant.get_mut_publisher(publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle)
                        else {
                            return;
                        };
                        if let Some(l) = data_writer.listener() {
                            l.send_actor_mail(data_writer_listener::TriggerPublicationMatched {
                                the_writer,
                                status,
                            });
                        }
                    } else if publisher
                        .listener_mask()
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) =
                            self.domain_participant.get_mut_publisher(publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_publication_matched_status();
                        if let Some(l) = publisher.listener() {
                            l.send_actor_mail(publisher_listener::TriggerOnPublicationMatched {
                                the_writer,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) =
                            self.domain_participant.get_mut_publisher(publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_publication_matched_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerPublicationMatched {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    }

                    let Some(publisher) =
                        self.domain_participant.get_mut_publisher(publisher_handle)
                    else {
                        return;
                    };
                    let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle)
                    else {
                        return;
                    };
                    data_writer.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::PublicationMatched,
                        },
                    );
                } else {
                    data_writer.add_incompatible_subscription(
                        InstanceHandle::new(
                            discovered_reader_data.dds_subscription_data.key().value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_writer
                        .listener_mask()
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let status = data_writer.get_offered_incompatible_qos_status();
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) =
                            self.domain_participant.get_mut_publisher(publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle)
                        else {
                            return;
                        };
                        if let Some(l) = data_writer.listener() {
                            l.send_actor_mail(
                                data_writer_listener::TriggerOfferedIncompatibleQos {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    } else if publisher
                        .listener_mask()
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) =
                            self.domain_participant.get_mut_publisher(publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_offered_incompatible_qos_status();
                        if let Some(l) = publisher.listener() {
                            l.send_actor_mail(publisher_listener::TriggerOfferedIncompatibleQos {
                                the_writer,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let Ok(the_writer) = self.get_data_writer_async(
                            participant_address,
                            publisher_handle,
                            data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) =
                            self.domain_participant.get_mut_publisher(publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_offered_incompatible_qos_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerOfferedIncompatibleQos {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    }

                    let Some(publisher) =
                        self.domain_participant.get_mut_publisher(publisher_handle)
                    else {
                        return;
                    };
                    let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle)
                    else {
                        return;
                    };
                    data_writer.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::OfferedIncompatibleQos,
                        },
                    );
                }
            }
        }
    }

    fn add_discovered_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == discovered_writer_data
                        .writer_proxy
                        .remote_writer_guid
                        .prefix()
            }) {
            p.participant_proxy.default_unicast_locator_list.clone()
        } else {
            vec![]
        };
        let default_multicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == discovered_writer_data
                        .writer_proxy
                        .remote_writer_guid
                        .prefix()
            }) {
            p.participant_proxy.default_multicast_locator_list.clone()
        } else {
            vec![]
        };
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return;
        };
        let is_any_name_matched = discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .any(|n| subscriber.qos().partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_writer_data
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

        let is_partition_matched = discovered_writer_data.dds_publication_data.partition
            == subscriber.qos().partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let subscriber_qos = subscriber.qos().clone();
            let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
                return;
            };
            let is_matched_topic_name = discovered_writer_data.dds_publication_data.topic_name()
                == data_reader.topic_name();
            let is_matched_type_name = discovered_writer_data.dds_publication_data.get_type_name()
                == data_reader.type_name();

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_writer_incompatible_qos_policy_list(
                        data_reader,
                        &discovered_writer_data.dds_publication_data,
                        &subscriber_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_reader.add_matched_publication(
                        discovered_writer_data.dds_publication_data.clone(),
                    );
                    let unicast_locator_list = if discovered_writer_data
                        .writer_proxy
                        .unicast_locator_list
                        .is_empty()
                    {
                        default_unicast_locator_list
                    } else {
                        discovered_writer_data.writer_proxy.unicast_locator_list
                    };
                    let multicast_locator_list = if discovered_writer_data
                        .writer_proxy
                        .multicast_locator_list
                        .is_empty()
                    {
                        default_multicast_locator_list
                    } else {
                        discovered_writer_data.writer_proxy.multicast_locator_list
                    };
                    let reliability_kind = match data_reader.qos().reliability.kind {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind = match data_reader.qos().durability.kind {
                        DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                        DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                        DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                        DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                    };
                    let writer_proxy = transport::reader::WriterProxy {
                        remote_writer_guid: discovered_writer_data.writer_proxy.remote_writer_guid,
                        remote_group_entity_id: discovered_writer_data
                            .writer_proxy
                            .remote_group_entity_id,
                        unicast_locator_list,
                        multicast_locator_list,
                        reliability_kind,
                        durability_kind,
                    };
                    if let TransportReaderKind::Stateful(r) = data_reader.transport_reader_mut() {
                        r.add_matched_writer(writer_proxy);
                    }

                    if data_reader
                        .listener_mask()
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = data_reader.listener() {
                            l.send_actor_mail(data_reader_listener::TriggerSubscriptionMatched {
                                the_reader,
                                status,
                            });
                        }
                    } else if subscriber
                        .listener_mask()
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = subscriber.listener() {
                            l.send_actor_mail(subscriber_listener::TriggerSubscriptionMatched {
                                the_reader,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerSubscriptionMatched {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .get_mut_subscriber(subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                    else {
                        return;
                    };
                    data_reader.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::SubscriptionMatched,
                        },
                    );
                } else {
                    data_reader.add_requested_incompatible_qos(
                        InstanceHandle::new(
                            discovered_writer_data.dds_publication_data.key().value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_reader
                        .listener_mask()
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let status = data_reader.get_requested_incompatible_qos_status();
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = data_reader.listener() {
                            l.send_actor_mail(
                                data_reader_listener::TriggerRequestedIncompatibleQos {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    } else if subscriber
                        .listener_mask()
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = subscriber.listener() {
                            l.send_actor_mail(
                                subscriber_listener::TriggerRequestedIncompatibleQos {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerRequestedIncompatibleQos {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .get_mut_subscriber(subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                    else {
                        return;
                    };
                    data_reader.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::RequestedIncompatibleQos,
                        },
                    );
                }
            }
        }
    }

    pub fn add_builtin_topics_detector_cache_change(&mut self, cache_change: CacheChange) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(topic_builtin_topic_data) =
                    TopicBuiltinTopicData::deserialize_data(cache_change.data_value.as_ref())
                {
                    self.domain_participant
                        .add_discovered_topic(topic_builtin_topic_data.clone());
                    for topic in self.domain_participant.topic_list_mut() {
                        if topic.topic_name() == topic_builtin_topic_data.name()
                            && topic.type_name() == topic_builtin_topic_data.get_type_name()
                            && !is_discovered_topic_consistent(
                                topic.qos(),
                                &topic_builtin_topic_data,
                            )
                        {
                            topic.increment_inconsistent_topic_status();
                        }
                    }
                }
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (),
        }

        let reception_timestamp = self.domain_participant.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_TOPIC)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    pub fn add_cache_change(
        &mut self,
        participant_address: ActorAddress<DomainParticipantActor>,
        cache_change: CacheChange,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let reception_timestamp = self.domain_participant.get_current_time();
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return;
        };

        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return;
        };
        let writer_instance_handle = InstanceHandle::new(cache_change.writer_guid.into());

        if data_reader
            .get_matched_publication_data(&writer_instance_handle)
            .is_some()
        {
            match data_reader.add_reader_change(cache_change, reception_timestamp) {
                Ok(AddChangeResult::Added(change_instance_handle)) => {
                    if let DurationKind::Finite(deadline_missed_period) =
                        data_reader.qos().deadline.period
                    {
                        let timer_handle = self.timer_driver.handle();
                        let participant_address = participant_address.clone();
                        let requested_deadline_missed_task =
                            self.backend_executor.handle().spawn(async move {
                                loop {
                                    timer_handle.sleep(deadline_missed_period.into()).await;
                                    participant_address
                                        .send_actor_mail(DomainParticipantMail::Event(
                                            EventServiceMail::RequestedDeadlineMissed {
                                                subscriber_handle: subscriber_handle,
                                                data_reader_handle: data_reader_handle,
                                                change_instance_handle,
                                                participant_address: participant_address.clone(),
                                            },
                                        ))
                                        .ok();
                                }
                            });

                        data_reader.insert_instance_deadline_missed_task(
                            change_instance_handle,
                            requested_deadline_missed_task,
                        );
                    }
                    let deta_reader_on_data_available_active = data_reader
                        .listener_mask()
                        .contains(&StatusKind::DataAvailable);

                    let Some(subscriber) = self
                        .domain_participant
                        .get_mut_subscriber(subscriber_handle)
                    else {
                        return;
                    };

                    if subscriber
                        .listener_mask()
                        .contains(&StatusKind::DataOnReaders)
                    {
                        let Ok(the_subscriber) = self
                            .get_subscriber_async(participant_address.clone(), subscriber_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };

                        if let Some(l) = subscriber.listener() {
                            l.send_actor_mail(subscriber_listener::TriggerDataOnReaders {
                                the_subscriber,
                            });
                        }
                    } else if deta_reader_on_data_available_active {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = data_reader.listener() {
                            l.send_actor_mail(data_reader_listener::TriggerDataAvailable {
                                the_reader,
                            });
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .get_mut_subscriber(subscriber_handle)
                    else {
                        return;
                    };

                    subscriber.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::DataOnReaders,
                        },
                    );
                    let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                    else {
                        return;
                    };
                    data_reader.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::DataAvailable,
                        },
                    );
                }
                Ok(AddChangeResult::NotAdded) => (), // Do nothing
                Ok(AddChangeResult::Rejected(instance_handle, sample_rejected_status_kind)) => {
                    data_reader.increment_sample_rejected_status(
                        instance_handle,
                        sample_rejected_status_kind,
                    );

                    if data_reader
                        .listener_mask()
                        .contains(&StatusKind::SampleRejected)
                    {
                        let status = data_reader.get_sample_rejected_status();
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = data_reader.listener() {
                            l.send_actor_mail(data_reader_listener::TriggerSampleRejected {
                                the_reader,
                                status,
                            });
                        }
                    } else if subscriber
                        .listener_mask()
                        .contains(&StatusKind::SampleRejected)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_sample_rejected_status();
                        if let Some(l) = subscriber.listener() {
                            l.send_actor_mail(subscriber_listener::TriggerSampleRejected {
                                status,
                                the_reader,
                            });
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::SampleRejected)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            participant_address,
                            subscriber_handle,
                            data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_sample_rejected_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(domain_participant_listener::TriggerSampleRejected {
                                status,
                                the_reader,
                            });
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .get_mut_subscriber(subscriber_handle)
                    else {
                        return;
                    };

                    let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                    else {
                        return;
                    };
                    data_reader.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::SampleRejected,
                        },
                    );
                }
                Err(_) => (),
            }
        }
    }

    pub fn remove_writer_change(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        sequence_number: i64,
    ) {
        if let Some(p) = self.domain_participant.get_mut_publisher(publisher_handle) {
            if let Some(dw) = p.get_mut_data_writer(data_writer_handle) {
                dw.remove_change(sequence_number);
            }
        }
    }

    pub fn offered_deadline_missed(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return;
        };

        data_writer.increment_offered_deadline_missed_status(change_instance_handle);

        if data_writer
            .listener_mask()
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let status = data_writer.get_offered_deadline_missed_status();
            let Ok(the_writer) = self.get_data_writer_async(
                participant_address,
                publisher_handle,
                data_writer_handle,
            ) else {
                return;
            };

            let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
                return;
            };

            if let Some(l) = data_writer.listener() {
                l.send_actor_mail(data_writer_listener::TriggerOfferedDeadlineMissed {
                    the_writer,
                    status,
                });
            }
        } else if publisher
            .listener_mask()
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let Ok(the_writer) = self.get_data_writer_async(
                participant_address,
                publisher_handle,
                data_writer_handle,
            ) else {
                return;
            };
            let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
                return;
            };
            let status = data_writer.get_offered_deadline_missed_status();
            if let Some(l) = publisher.listener() {
                l.send_actor_mail(publisher_listener::TriggerOfferedDeadlineMissed {
                    the_writer,
                    status,
                });
            }
        } else if self
            .domain_participant
            .listener_mask()
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let Ok(the_writer) = self.get_data_writer_async(
                participant_address,
                publisher_handle,
                data_writer_handle,
            ) else {
                return;
            };

            let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle)
            else {
                return;
            };
            let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
                return;
            };
            let status = data_writer.get_offered_deadline_missed_status();
            if let Some(l) = self.domain_participant.listener() {
                l.send_actor_mail(domain_participant_listener::TriggerOfferedDeadlineMissed {
                    the_writer,
                    status,
                });
            }
        }

        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return;
        };
        data_writer.status_condition().send_actor_mail(
            status_condition_actor::AddCommunicationState {
                state: StatusKind::OfferedDeadlineMissed,
            },
        );
    }

    pub fn requested_deadline_missed(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return;
        };
        data_reader.remove_instance_ownership(&change_instance_handle);
        data_reader.increment_requested_deadline_missed_status(change_instance_handle);

        if data_reader
            .listener_mask()
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let status = data_reader.get_requested_deadline_missed_status();
            let Ok(the_reader) = self.get_data_reader_async(
                participant_address,
                subscriber_handle,
                data_reader_handle,
            ) else {
                return;
            };
            let Some(subscriber) = self
                .domain_participant
                .get_mut_subscriber(subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
                return;
            };
            if let Some(l) = data_reader.listener() {
                l.send_actor_mail(data_reader_listener::TriggerRequestedDeadlineMissed {
                    the_reader,
                    status,
                });
            }
        } else if subscriber
            .listener_mask()
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let Ok(the_reader) = self.get_data_reader_async(
                participant_address,
                subscriber_handle,
                data_reader_handle,
            ) else {
                return;
            };

            let Some(subscriber) = self
                .domain_participant
                .get_mut_subscriber(subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
                return;
            };
            let status = data_reader.get_requested_deadline_missed_status();
            if let Some(l) = subscriber.listener() {
                l.send_actor_mail(subscriber_listener::TriggerRequestedDeadlineMissed {
                    status,
                    the_reader,
                });
            }
        } else if self
            .domain_participant
            .listener_mask()
            .contains(&StatusKind::RequestedDeadlineMissed)
        {
            let Ok(the_reader) = self.get_data_reader_async(
                participant_address,
                subscriber_handle,
                data_reader_handle,
            ) else {
                return;
            };

            let Some(subscriber) = self
                .domain_participant
                .get_mut_subscriber(subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
                return;
            };
            let status = data_reader.get_requested_deadline_missed_status();
            if let Some(l) = self.domain_participant.listener() {
                l.send_actor_mail(
                    domain_participant_listener::TriggerRequestedDeadlineMissed {
                        status,
                        the_reader,
                    },
                );
            }
        }
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return;
        };

        data_reader.status_condition().send_actor_mail(
            status_condition_actor::AddCommunicationState {
                state: StatusKind::RequestedDeadlineMissed,
            },
        );
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

fn get_discovered_reader_incompatible_qos_policy_list(
    writer_qos: &DataWriterQos,
    discovered_reader_data: &SubscriptionBuiltinTopicData,
    publisher_qos: &PublisherQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();
    if &writer_qos.durability < discovered_reader_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if publisher_qos.presentation.access_scope < discovered_reader_data.presentation().access_scope
        || publisher_qos.presentation.coherent_access
            != discovered_reader_data.presentation().coherent_access
        || publisher_qos.presentation.ordered_access
            != discovered_reader_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &writer_qos.deadline > discovered_reader_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &writer_qos.latency_budget < discovered_reader_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &writer_qos.liveliness < discovered_reader_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if writer_qos.reliability.kind < discovered_reader_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &writer_qos.destination_order < discovered_reader_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if writer_qos.ownership.kind != discovered_reader_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = writer_qos
        .representation
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !(discovered_reader_data
        .representation()
        .value
        .contains(writer_offered_representation)
        || (writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && discovered_reader_data.representation().value.is_empty()))
    {
        incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID);
    }

    incompatible_qos_policy_list
}

fn get_discovered_writer_incompatible_qos_policy_list(
    data_reader: &DataReaderEntity,
    publication_builtin_topic_data: &PublicationBuiltinTopicData,
    subscriber_qos: &SubscriberQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();

    if subscriber_qos.presentation.access_scope
        > publication_builtin_topic_data.presentation().access_scope
        || subscriber_qos.presentation.coherent_access
            != publication_builtin_topic_data
                .presentation()
                .coherent_access
        || subscriber_qos.presentation.ordered_access
            != publication_builtin_topic_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &data_reader.qos().durability > publication_builtin_topic_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos().deadline < publication_builtin_topic_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &data_reader.qos().latency_budget > publication_builtin_topic_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &data_reader.qos().liveliness > publication_builtin_topic_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if data_reader.qos().reliability.kind > publication_builtin_topic_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos().destination_order > publication_builtin_topic_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if data_reader.qos().ownership.kind != publication_builtin_topic_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = publication_builtin_topic_data
        .representation()
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !data_reader
        .qos()
        .representation
        .value
        .contains(writer_offered_representation)
    {
        // Empty list is interpreted as containing XCDR_DATA_REPRESENTATION
        if !(writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && data_reader.qos().representation.value.is_empty())
        {
            incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID)
        }
    }

    incompatible_qos_policy_list
}

fn is_discovered_topic_consistent(
    topic_qos: &TopicQos,
    topic_builtin_topic_data: &TopicBuiltinTopicData,
) -> bool {
    &topic_qos.topic_data == topic_builtin_topic_data.topic_data()
        && &topic_qos.durability == topic_builtin_topic_data.durability()
        && &topic_qos.deadline == topic_builtin_topic_data.deadline()
        && &topic_qos.latency_budget == topic_builtin_topic_data.latency_budget()
        && &topic_qos.liveliness == topic_builtin_topic_data.liveliness()
        && &topic_qos.reliability == topic_builtin_topic_data.reliability()
        && &topic_qos.destination_order == topic_builtin_topic_data.destination_order()
        && &topic_qos.history == topic_builtin_topic_data.history()
        && &topic_qos.resource_limits == topic_builtin_topic_data.resource_limits()
        && &topic_qos.transport_priority == topic_builtin_topic_data.transport_priority()
        && &topic_qos.lifespan == topic_builtin_topic_data.lifespan()
        && &topic_qos.ownership == topic_builtin_topic_data.ownership()
}
