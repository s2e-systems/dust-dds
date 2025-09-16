use super::domain_participant_mail::{DcpsDomainParticipantMail, EventServiceMail, MessageServiceMail};
use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    dcps::{
        actor::{Actor, ActorAddress},
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
            spdp_discovered_participant_data::{
                BuiltinEndpointQos, BuiltinEndpointSet, ParticipantProxy,
                SpdpDiscoveredParticipantData,
            },
        },
        domain_participant_factory::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        },
        listeners::domain_participant_listener::ListenerMail,
        status_condition::DcpsStatusCondition,
        status_condition_mail::DcpsStatusConditionMail,
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
            get_serialized_key_from_serialized_foo,
        },
    },
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
        domain_participant::DomainParticipantAsync, publisher::PublisherAsync,
        subscriber::SubscriberAsync, topic::TopicAsync, topic_description::TopicDescriptionAsync,
    },
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, QosKind,
            SubscriberQos, TopicQos,
        },
        qos_policy::{
            DestinationOrderQosPolicyKind, DurabilityQosPolicyKind, HistoryQosPolicy,
            HistoryQosPolicyKind, Length, LifespanQosPolicy, OwnershipQosPolicyKind, QosPolicyId,
            ReliabilityQosPolicyKind, ResourceLimitsQosPolicy, TransportPriorityQosPolicy,
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
        status::{
            InconsistentTopicStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleRejectedStatus, SampleRejectedStatusKind,
            StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind, Time},
        type_support::{DdsDeserialize, DdsSerialize},
    },
    runtime::{ChannelSend, Clock, DdsRuntime, OneshotReceive, Spawner, Timer},
    transport::{
        self,
        interface::{
            HistoryCache, TransportParticipant, TransportParticipantFactory,
            TransportStatefulReader, TransportStatefulWriter, TransportStatelessReader,
            TransportStatelessWriter,
        },
        types::{
            CacheChange, ChangeKind, DurabilityKind, EntityId, Guid, ReliabilityKind, TopicKind,
            ENTITYID_UNKNOWN, USER_DEFINED_READER_NO_KEY, USER_DEFINED_READER_WITH_KEY,
            USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY,
        },
    },
    xtypes::dynamic_type::DynamicType,
};
use alloc::{
    boxed::Box,
    collections::VecDeque,
    format,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
use core::{
    future::{poll_fn, Future},
    pin::{pin, Pin},
    task::Poll,
};
use regex::Regex;

pub fn poll_timeout<T>(
    mut timer_handle: impl Timer,
    duration: core::time::Duration,
    mut future: Pin<Box<dyn Future<Output = T> + Send>>,
) -> impl Future<Output = DdsResult<T>> {
    poll_fn(move |cx| {
        let timeout = timer_handle.delay(duration);
        if let Poll::Ready(t) = pin!(&mut future).poll(cx) {
            return Poll::Ready(Ok(t));
        }
        if pin!(timeout).poll(cx).is_ready() {
            return Poll::Ready(Err(DdsError::Timeout));
        }

        Poll::Pending
    })
}

pub struct DcpsDomainParticipant<R: DdsRuntime, T: TransportParticipantFactory> {
    pub transport: T::TransportParticipant,
    pub instance_handle_counter: InstanceHandleCounter,
    pub entity_counter: u16,
    pub domain_participant: DomainParticipantEntity<R, T>,
    pub clock_handle: R::ClockHandle,
    pub timer_handle: R::TimerHandle,
    pub spawner_handle: R::SpawnerHandle,
}

impl<R, T> DcpsDomainParticipant<R, T>
where
    R: DdsRuntime,
    T: TransportParticipantFactory,
{
    pub fn new(
        domain_participant: DomainParticipantEntity<R, T>,
        transport: T::TransportParticipant,
        instance_handle_counter: InstanceHandleCounter,
        clock_handle: R::ClockHandle,
        timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
    ) -> Self {
        Self {
            transport,
            instance_handle_counter,
            entity_counter: 0,
            domain_participant,
            clock_handle,
            timer_handle,
            spawner_handle,
        }
    }

    fn get_participant_async(
        &self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) -> DomainParticipantAsync<R> {
        DomainParticipantAsync::new(
            participant_address,
            self.domain_participant
                .builtin_subscriber()
                .status_condition
                .address(),
            self.domain_participant.domain_id,
            self.domain_participant.instance_handle,
            self.spawner_handle.clone(),
            self.clock_handle.clone(),
            self.timer_handle.clone(),
        )
    }

    fn get_subscriber_async(
        &self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<SubscriberAsync<R>> {
        Ok(SubscriberAsync::new(
            subscriber_handle,
            self.domain_participant
                .user_defined_subscriber_list
                .iter()
                .find(|x| x.instance_handle == subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .status_condition
                .address(),
            self.get_participant_async(participant_address),
        ))
    }

    fn get_data_reader_async<Foo>(
        &self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<DataReaderAsync<R, Foo>> {
        let data_reader = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(DataReaderAsync::new(
            data_reader_handle,
            data_reader.status_condition.address(),
            self.get_subscriber_async(participant_address.clone(), subscriber_handle)?,
            self.get_topic_description_async(participant_address, data_reader.topic_name.clone())?,
        ))
    }

    fn get_publisher_async(
        &self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<PublisherAsync<R>> {
        Ok(PublisherAsync::new(
            publisher_handle,
            self.get_participant_async(participant_address),
        ))
    }

    fn get_data_writer_async<Foo>(
        &self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<DataWriterAsync<R, Foo>> {
        let data_writer = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(DataWriterAsync::new(
            data_writer_handle,
            data_writer.status_condition.address(),
            self.get_publisher_async(participant_address.clone(), publisher_handle)?,
            self.get_topic_description_async(participant_address, data_writer.topic_name.clone())?,
        ))
    }

    fn get_topic_description_async(
        &self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        topic_name: String,
    ) -> DdsResult<TopicDescriptionAsync<R>> {
        let topic = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;
        Ok(TopicDescriptionAsync::Topic(TopicAsync::new(
            topic.instance_handle,
            topic.status_condition.address(),
            topic.type_name.clone(),
            topic_name,
            self.get_participant_async(participant_address),
        )))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_inconsistent_topic_status(
        &mut self,
        topic_name: String,
    ) -> DdsResult<InconsistentTopicStatus> {
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter_mut()
            .find(|x| x.topic_name == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(topic.get_inconsistent_topic_status().await)
    }

    #[tracing::instrument(skip(self))]
    pub fn set_topic_qos(
        &mut self,
        topic_name: String,
        topic_qos: QosKind<TopicQos>,
    ) -> DdsResult<()> {
        let qos = match topic_qos {
            QosKind::Default => self.domain_participant.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter_mut()
            .find(|x| x.topic_name == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;

        if topic.enabled
            && (topic.qos.durability != qos.durability
                || topic.qos.liveliness != qos.liveliness
                || topic.qos.reliability != qos.reliability
                || topic.qos.destination_order != qos.destination_order
                || topic.qos.history != qos.history
                || topic.qos.resource_limits != qos.resource_limits
                || topic.qos.ownership != qos.ownership)
        {
            return Err(DdsError::ImmutablePolicy);
        }

        topic.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_topic_qos(&mut self, topic_name: String) -> DdsResult<TopicQos> {
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter_mut()
            .find(|x| x.topic_name == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(topic.qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable_topic(&mut self, topic_name: String) -> DdsResult<()> {
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter_mut()
            .find(|x| x.topic_name == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !topic.enabled {
            topic.enable();
            self.announce_topic(topic_name).await;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_type_support(
        &mut self,
        topic_name: String,
    ) -> DdsResult<Arc<dyn DynamicType + Send + Sync>> {
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter_mut()
            .find(|x| x.topic_name == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(topic.type_support.clone())
    }

    #[tracing::instrument(skip(self, listener_sender))]
    pub fn create_user_defined_publisher(
        &mut self,
        qos: QosKind<PublisherQos>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<InstanceHandle> {
        let publisher_qos = match qos {
            QosKind::Default => self.domain_participant.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let publisher_handle = self.instance_handle_counter.generate_new_instance_handle();
        let data_writer_list = Default::default();
        let mut publisher = PublisherEntity::new(
            publisher_qos,
            publisher_handle,
            data_writer_list,
            listener_sender,
            mask,
        );

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            publisher.enable();
        }

        self.domain_participant
            .user_defined_publisher_list
            .push(publisher);

        Ok(publisher_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_publisher(
        &mut self,
        participant_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<()> {
        if participant_handle != self.domain_participant.instance_handle {
            return Err(DdsError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant".to_string(),
            ));
        }
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
        else {
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

    #[tracing::instrument(skip(self, status_condition, listener_sender))]
    pub fn create_user_defined_subscriber(
        &mut self,
        qos: QosKind<SubscriberQos>,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<InstanceHandle> {
        let subscriber_qos = match qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let subscriber_handle = self.instance_handle_counter.generate_new_instance_handle();

        let listener_mask = mask.to_vec();
        let data_reader_list = Default::default();
        let mut subscriber = SubscriberEntity::new(
            subscriber_handle,
            subscriber_qos,
            data_reader_list,
            status_condition,
            listener_sender,
            listener_mask,
        );

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            subscriber.enable();
        }

        self.domain_participant
            .user_defined_subscriber_list
            .push(subscriber);

        Ok(subscriber_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_subscriber(
        &mut self,
        participant_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<()> {
        if self.domain_participant.instance_handle != participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Subscriber can only be deleted from its parent participant".to_string(),
            ));
        }

        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !subscriber.data_reader_list.is_empty() {
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

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, status_condition, listener_sender, type_support))]
    pub async fn create_topic(
        &mut self,
        topic_name: String,
        type_name: String,
        qos: QosKind<TopicQos>,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
        type_support: Arc<dyn DynamicType + Send + Sync>,
    ) -> DdsResult<InstanceHandle> {
        if self
            .domain_participant
            .topic_list
            .iter()
            .any(|x| x.topic_name == topic_name)
        {
            return Err(DdsError::PreconditionNotMet(format!(
                "Topic with name {topic_name} already exists.
         To access this topic call the lookup_topicdescription method.",
            )));
        }

        let qos = match qos {
            QosKind::Default => self.domain_participant.default_topic_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let topic_handle = self.instance_handle_counter.generate_new_instance_handle();

        let topic = TopicEntity::new(
            qos,
            type_name,
            topic_name.clone(),
            topic_handle,
            status_condition,
            listener_sender,
            mask,
            type_support,
        );

        match self
            .domain_participant
            .topic_list
            .iter_mut()
            .find(|x| x.topic_name == topic.topic_name)
        {
            Some(x) => *x = topic,
            None => self.domain_participant.topic_list.push(topic),
        }

        if self.domain_participant.enabled
            && self
                .domain_participant
                .qos
                .entity_factory
                .autoenable_created_entities
        {
            self.enable_topic(topic_name).await?;
        }

        Ok(topic_handle)
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_user_defined_topic(
        &mut self,
        participant_handle: InstanceHandle,
        topic_name: String,
    ) -> DdsResult<()> {
        if self.domain_participant.instance_handle != participant_handle {
            return Err(DdsError::PreconditionNotMet(
                "Topic can only be deleted from its parent participant".to_string(),
            ));
        }

        if BUILT_IN_TOPIC_NAME_LIST.contains(&topic_name.as_str()) {
            return Ok(());
        }

        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if Arc::strong_count(&topic.type_support) > 1 {
            return Err(DdsError::PreconditionNotMet(
                "Topic still attached to some data writer or data reader".to_string(),
            ));
        }

        for content_filtered_topic in self.domain_participant.content_filtered_topic_list.iter() {
            if content_filtered_topic.related_topic_name == topic_name {
                return Err(DdsError::PreconditionNotMet(
                    "Topic still attached to content filtered topic".to_string(),
                ));
            }
        }

        if let Some(index) = self
            .domain_participant
            .topic_list
            .iter()
            .position(|x| x.topic_name == topic_name)
        {
            self.domain_participant.topic_list.remove(index)
        } else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn create_content_filtered_topic(
        &mut self,
        participant_handle: InstanceHandle,
        name: String,
        related_topic_name: String,
    ) -> DdsResult<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn delete_content_filtered_topic(
        &mut self,
        participant_handle: InstanceHandle,
        name: String,
    ) -> DdsResult<()> {
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip(self, type_support, status_condition))]
    pub fn find_topic(
        &mut self,
        topic_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
    ) -> DdsResult<
        Option<(
            InstanceHandle,
            ActorAddress<R, DcpsStatusCondition<R>>,
            String,
        )>,
    > {
        if let Some(topic) = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
        {
            Ok(Some((
                topic.instance_handle,
                topic.status_condition.address(),
                topic.type_name.clone(),
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
                    status_condition,
                    None,
                    vec![],
                    type_support,
                );
                topic.enable();
                let topic_status_condition_address = topic.status_condition.address();

                match self
                    .domain_participant
                    .topic_list
                    .iter_mut()
                    .find(|x| x.topic_name == topic.topic_name)
                {
                    Some(x) => *x = topic,
                    None => self.domain_participant.topic_list.push(topic),
                }

                return Ok(Some((
                    topic_handle,
                    topic_status_condition_address,
                    type_name,
                )));
            }
            Ok(None)
        }
    }

    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub fn lookup_topicdescription(
        &mut self,
        topic_name: String,
    ) -> DdsResult<
        Option<(
            String,
            InstanceHandle,
            ActorAddress<R, DcpsStatusCondition<R>>,
        )>,
    > {
        if let Some(topic) = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
        {
            Ok(Some((
                topic.type_name.clone(),
                topic.instance_handle,
                topic.status_condition.address(),
            )))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn ignore_participant(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.domain_participant.enabled {
            if !self
                .domain_participant
                .ignored_participants
                .contains(&handle)
            {
                self.domain_participant.ignored_participants.push(handle);
            }

            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn ignore_subscription(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.domain_participant.enabled {
            if !self
                .domain_participant
                .ignored_subcriptions
                .contains(&handle)
            {
                self.domain_participant.ignored_subcriptions.push(handle);
            }
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn ignore_publication(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        if self.domain_participant.enabled {
            if !self
                .domain_participant
                .ignored_publications
                .contains(&handle)
            {
                self.domain_participant.ignored_publications.push(handle);
            }
            Ok(())
        } else {
            Err(DdsError::NotEnabled)
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_participant_contained_entities(&mut self) -> DdsResult<()> {
        let deleted_publisher_list: Vec<PublisherEntity<R, T>> = self
            .domain_participant
            .user_defined_publisher_list
            .drain(..)
            .collect();
        for mut publisher in deleted_publisher_list {
            for data_writer in publisher.drain_data_writer_list() {
                self.announce_deleted_data_writer(data_writer).await;
            }
        }

        let deleted_subscriber_list: Vec<SubscriberEntity<R, T>> = self
            .domain_participant
            .user_defined_subscriber_list
            .drain(..)
            .collect();
        for mut subscriber in deleted_subscriber_list {
            for data_reader in subscriber.data_reader_list.drain(..) {
                self.announce_deleted_data_reader(data_reader).await;
            }
        }

        self.domain_participant
            .topic_list
            .retain(|x| BUILT_IN_TOPIC_NAME_LIST.contains(&x.topic_name.as_ref()));

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_publisher_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => PublisherQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.default_publisher_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_publisher_qos(&mut self) -> DdsResult<PublisherQos> {
        Ok(self.domain_participant.default_publisher_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_subscriber_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => SubscriberQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.default_subscriber_qos = qos;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_subscriber_qos(&mut self) -> DdsResult<SubscriberQos> {
        Ok(self.domain_participant.default_subscriber_qos.clone())
    }

    #[tracing::instrument(skip(self))]
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

        qos.is_consistent()?;
        self.domain_participant.default_topic_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.domain_participant.default_topic_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_participants(&mut self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .domain_participant
            .discovered_participant_list
            .iter()
            .map(|p| InstanceHandle::new(p.dds_participant_data.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_participant_data(
        &mut self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        let Some(handle) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| &p.dds_participant_data.key().value == participant_handle.as_ref())
        else {
            return Err(DdsError::BadParameter);
        };
        Ok(handle.dds_participant_data.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_topics(&mut self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self
            .domain_participant
            .discovered_topic_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_discovered_topic_data(
        &mut self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        let Some(handle) = self
            .domain_participant
            .get_discovered_topic_data(&topic_handle)
        else {
            return Err(DdsError::PreconditionNotMet(String::from(
                "Topic with this handle not discovered",
            )));
        };

        Ok(handle.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_current_time(&mut self) -> Time {
        self.clock_handle.now()
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_domain_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.domain_participant.qos = qos;
        if self.domain_participant.enabled {
            self.announce_participant().await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_domain_participant_qos(&mut self) -> DdsResult<DomainParticipantQos> {
        Ok(self.domain_participant.qos.clone())
    }

    #[tracing::instrument(skip(self, listener_sender))]
    pub fn set_domain_participant_listener(
        &mut self,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<()> {
        self.domain_participant.listener_sender = listener_sender;
        self.domain_participant.listener_mask = status_kind;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable_domain_participant(&mut self) -> DdsResult<()> {
        if !self.domain_participant.enabled {
            self.domain_participant.enabled = true;

            self.announce_participant().await;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn is_participant_empty(&mut self) -> bool {
        self.domain_participant.is_empty()
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(
        self,
        status_condition,
        listener_sender,
        domain_participant_address
    ))]
    pub async fn create_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataReaderQos>,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
        domain_participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) -> DdsResult<InstanceHandle> {
        struct UserDefinedReaderHistoryCache<R>
        where
            R: DdsRuntime,
        {
            pub domain_participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
            pub subscriber_handle: InstanceHandle,
            pub data_reader_handle: InstanceHandle,
        }

        impl<R> HistoryCache for UserDefinedReaderHistoryCache<R>
        where
            R: DdsRuntime,
        {
            fn add_change(
                &mut self,
                cache_change: CacheChange,
            ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
                Box::pin(async move {
                    self.domain_participant_address
                        .send(DcpsDomainParticipantMail::Message(
                            MessageServiceMail::AddCacheChange {
                                participant_address: self.domain_participant_address.clone(),
                                cache_change,
                                subscriber_handle: self.subscriber_handle,
                                data_reader_handle: self.data_reader_handle,
                            },
                        ))
                        .await
                        .ok();
                })
            }

            fn remove_change(
                &mut self,
                _sequence_number: i64,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
                todo!()
            }
        }

        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic_kind = get_topic_kind(topic.type_support.as_ref());
        let topic_name = topic.topic_name.clone();
        let type_name = topic.type_name.clone();
        let reader_handle = self.instance_handle_counter.generate_new_instance_handle();

        let type_support = topic.type_support.clone();
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos.clone(),
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
        let transport_reader = TransportReaderKind::Stateful(
            self.transport
                .create_stateful_reader(
                    entity_id,
                    reliablity_kind,
                    Box::new(UserDefinedReaderHistoryCache::<R> {
                        domain_participant_address: domain_participant_address.clone(),
                        subscriber_handle: subscriber.instance_handle,
                        data_reader_handle: reader_handle,
                    }),
                )
                .await,
        );

        let listener_mask = mask.to_vec();
        let data_reader = DataReaderEntity::new(
            reader_handle,
            qos,
            topic_name,
            type_name,
            type_support,
            status_condition,
            listener_sender,
            listener_mask,
            transport_reader,
        );

        let data_reader_handle = data_reader.instance_handle;

        subscriber.data_reader_list.push(data_reader);

        if subscriber.enabled && subscriber.qos.entity_factory.autoenable_created_entities {
            self.enable_data_reader(
                subscriber_handle,
                data_reader_handle,
                domain_participant_address,
            )
            .await?;
        }
        Ok(data_reader_handle)
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        datareader_handle: InstanceHandle,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x: &&mut SubscriberEntity<R, T>| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if let Some(index) = subscriber
            .data_reader_list
            .iter()
            .position(|x| x.instance_handle == datareader_handle)
        {
            let data_reader = subscriber.data_reader_list.remove(index);
            self.announce_deleted_data_reader(data_reader).await;
        } else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub fn lookup_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        topic_name: String,
    ) -> DdsResult<Option<(InstanceHandle, ActorAddress<R, DcpsStatusCondition<R>>)>> {
        if !self
            .domain_participant
            .topic_list
            .iter()
            .any(|x| x.topic_name == topic_name)
        {
            return Err(DdsError::BadParameter);
        }

        // Built-in subscriber is identified by the handle of the participant itself
        if self.domain_participant.instance_handle == subscriber_handle {
            Ok(self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| dr.topic_name == topic_name)
                .map(|x| (x.instance_handle, x.status_condition.address())))
        } else {
            let Some(s) = self
                .domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
            else {
                return Err(DdsError::AlreadyDeleted);
            };
            Ok(s.data_reader_list
                .iter_mut()
                .find(|dr| dr.topic_name == topic_name)
                .map(|x| (x.instance_handle, x.status_condition.address())))
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn set_default_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => DataReaderQos::default(),
            QosKind::Specific(q) => q,
        };
        qos.is_consistent()?;
        subscriber.default_data_reader_qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<DataReaderQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(subscriber.default_data_reader_qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn set_subscriber_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        qos: QosKind<SubscriberQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_subscriber_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if subscriber.enabled {
            subscriber.qos.check_immutability(&qos)?;
        }
        subscriber.qos = qos;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_subscriber_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<SubscriberQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(subscriber.qos.clone())
    }

    #[tracing::instrument(skip(self, listener_sender))]
    pub fn set_subscriber_listener(
        &mut self,
        subscriber_handle: InstanceHandle,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        subscriber.listener_sender = listener_sender;
        subscriber.listener_mask = mask;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self, status_condition, listener_sender, participant_address))]
    pub async fn create_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) -> DdsResult<InstanceHandle> {
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let topic_kind = get_topic_kind(topic.type_support.as_ref());
        let type_support = topic.type_support.clone();
        let type_name = topic.type_name.clone();
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
            .create_stateful_writer(entity_id, reliablity_kind)
            .await;

        let data_writer = DataWriterEntity::new(
            writer_handle,
            TransportWriterKind::Stateful(transport_writer),
            topic_name,
            type_name,
            type_support,
            status_condition,
            listener_sender,
            mask,
            qos,
        );
        let data_writer_handle = data_writer.instance_handle;

        publisher.insert_data_writer(data_writer);

        if publisher.enabled && publisher.qos.entity_factory.autoenable_created_entities {
            self.enable_data_writer(publisher_handle, writer_handle, participant_address)
                .await?;
        }

        Ok(data_writer_handle)
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_data_writer(
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
        self.announce_deleted_data_writer(data_writer).await;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_default_datawriter_qos(
        &mut self,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(publisher.default_datawriter_qos().clone())
    }

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
    pub fn set_publisher_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        qos: QosKind<PublisherQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => self.domain_participant.default_publisher_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        publisher.set_qos(qos)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_publisher_qos(
        &mut self,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<PublisherQos> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(publisher.qos.clone())
    }

    #[tracing::instrument(skip(self, listener_sender))]
    pub fn set_publisher_listener(
        &mut self,
        publisher_handle: InstanceHandle,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        publisher.set_listener(listener_sender, mask);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_publication_matched_status(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<PublicationMatchedStatus> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let status = data_writer.get_publication_matched_status();

        data_writer
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            })
            .await;
        Ok(status)
    }

    #[tracing::instrument(skip(self, listener_sender))]
    pub fn set_listener_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Ok(());
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return Ok(());
        };

        data_writer.listener_sender = listener_sender;
        data_writer.listener_mask = listener_mask;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_data_writer_qos(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<DataWriterQos> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.qos.clone())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_subscriptions(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_writer
            .matched_subscription_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect())
    }

    #[tracing::instrument(skip(self))]
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
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_writer
            .matched_subscription_list
            .iter()
            .find(|x| subscription_handle.as_ref() == &x.key().value)
            .ok_or(DdsError::BadParameter)
            .cloned()
    }

    #[tracing::instrument(skip(self))]
    pub async fn unregister_instance(
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
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let serialized_key = match get_serialized_key_from_serialized_foo(
            &serialized_data,
            data_writer.type_support.as_ref(),
        ) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };
        data_writer
            .unregister_w_timestamp(serialized_key, timestamp)
            .await
    }

    #[tracing::instrument(skip(self))]
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
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        if !data_writer.enabled {
            return Err(DdsError::NotEnabled);
        }

        let instance_handle = match get_instance_handle_from_serialized_foo(
            &serialized_data,
            data_writer.type_support.as_ref(),
        ) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(data_writer
            .registered_instance_list
            .contains(&instance_handle)
            .then_some(instance_handle))
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn write_w_timestamp(
        &mut self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        serialized_data: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let now = self.get_current_time();
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let instance_handle = match get_instance_handle_from_serialized_foo(
            &serialized_data,
            data_writer.type_support.as_ref(),
        ) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };

        match data_writer.qos.lifespan.duration {
            DurationKind::Finite(lifespan_duration) => {
                let mut timer_handle = self.timer_handle.clone();
                let sleep_duration = timestamp - now + lifespan_duration;
                if sleep_duration > Duration::new(0, 0) {
                    let sequence_number = match data_writer
                        .write_w_timestamp(serialized_data, timestamp, &self.clock_handle)
                        .await
                    {
                        Ok(s) => s,
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    let participant_address = participant_address.clone();
                    self.spawner_handle.spawn(async move {
                        timer_handle.delay(sleep_duration.into()).await;
                        participant_address
                            .send(DcpsDomainParticipantMail::Message(
                                MessageServiceMail::RemoveWriterChange {
                                    publisher_handle,
                                    data_writer_handle,
                                    sequence_number,
                                },
                            ))
                            .await
                            .ok();
                    });
                }
            }
            DurationKind::Infinite => {
                match data_writer
                    .write_w_timestamp(serialized_data, timestamp, &self.clock_handle)
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(e);
                    }
                };
            }
        }

        if let DurationKind::Finite(deadline_missed_period) = data_writer.qos.deadline.period {
            let mut timer_handle = self.timer_handle.clone();
            self.spawner_handle.spawn(async move {
                loop {
                    timer_handle.delay(deadline_missed_period.into()).await;
                    participant_address
                        .send(DcpsDomainParticipantMail::Event(
                            EventServiceMail::OfferedDeadlineMissed {
                                publisher_handle,
                                data_writer_handle,
                                change_instance_handle: instance_handle,
                                participant_address: participant_address.clone(),
                            },
                        ))
                        .await
                        .ok();
                }
            });
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn dispose_w_timestamp(
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
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let serialized_key = match get_serialized_key_from_serialized_foo(
            &serialized_data,
            data_writer.type_support.as_ref(),
        ) {
            Ok(k) => k,
            Err(e) => {
                return Err(e.into());
            }
        };
        data_writer
            .dispose_w_timestamp(serialized_key, timestamp)
            .await
    }

    //#[tracing::instrument(skip(self, participant_address))]
    pub fn wait_for_acknowledgments(
        &mut self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = DdsResult<()>> + Send>> {
        let timer_handle = self.timer_handle.clone();
        Box::pin(async move {
            poll_timeout(
                timer_handle,
                timeout.into(),
                Box::pin(async move {
                    loop {
                        let (reply_sender, reply_receiver) = R::oneshot();
                        participant_address
                            .send(DcpsDomainParticipantMail::Message(
                                MessageServiceMail::AreAllChangesAcknowledged {
                                    publisher_handle,
                                    data_writer_handle,
                                    reply_sender,
                                },
                            ))
                            .await
                            .ok();
                        let reply = reply_receiver.receive().await;
                        match reply {
                            Ok(are_changes_acknowledged) => match are_changes_acknowledged {
                                Ok(true) => return Ok(()),
                                Ok(false) => (),
                                Err(e) => return Err(e),
                            },
                            Err(_) => return Err(DdsError::Error(String::from("Channel error"))),
                        }
                    }
                }),
            )
            .await?
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_offered_deadline_missed_status(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_writer.get_offered_deadline_missed_status().await)
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn enable_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) -> DdsResult<()> {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_writer.enabled {
            data_writer.enable();

            let discovered_reader_list: Vec<_> =
                self.domain_participant.discovered_reader_list.to_vec();
            for discovered_reader_data in discovered_reader_list {
                self.add_discovered_reader(
                    discovered_reader_data,
                    publisher_handle,
                    data_writer_handle,
                    participant_address.clone(),
                )
                .await;
            }

            self.announce_data_writer(publisher_handle, data_writer_handle)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_data_writer_qos(
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
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;
        if data_writer.enabled {
            data_writer.qos.check_immutability(&qos)?;
        }
        data_writer.qos = qos;

        if data_writer.enabled {
            self.announce_data_writer(publisher_handle, data_writer_handle)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn are_all_changes_acknowledged(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_writer) = publisher.get_data_writer(data_writer_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_writer.are_all_changes_acknowledged().await)
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn read(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<Arc<[u8]>>, SampleInfo)>> {
        let subscriber = if subscriber_handle == self.domain_participant.instance_handle {
            Some(&mut self.domain_participant.builtin_subscriber)
        } else {
            self.domain_participant
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
        };

        let Some(subscriber) = subscriber else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        data_reader
            .read(
                max_samples,
                &sample_states,
                &view_states,
                &instance_states,
                specific_instance_handle,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn take(
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
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .take(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn read_next_instance(
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
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .read_next_instance(
                max_samples,
                previous_handle,
                &sample_states,
                &view_states,
                &instance_states,
            )
            .await
    }

    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[tracing::instrument(skip(self))]
    pub async fn take_next_instance(
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
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader
            .take_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_subscription_matched_status(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionMatchedStatus> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        let status = data_reader.get_subscription_matched_status();
        data_reader
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::SubscriptionMatched,
            })
            .await;
        Ok(status)
    }

    //#[tracing::instrument(skip(self, participant_address))]
    pub fn wait_for_historical_data(
        &mut self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        max_wait: Duration,
    ) -> Pin<Box<dyn Future<Output = DdsResult<()>> + Send>> {
        let timer_handle = self.timer_handle.clone();
        Box::pin(async move {
            poll_timeout(
                timer_handle,
                max_wait.into(),
                Box::pin(async move {
                    loop {
                        let (reply_sender, reply_receiver) = R::oneshot();
                        participant_address
                            .send(DcpsDomainParticipantMail::Message(
                                MessageServiceMail::IsHistoricalDataReceived {
                                    subscriber_handle,
                                    data_reader_handle,
                                    reply_sender,
                                },
                            ))
                            .await?;

                        let reply = reply_receiver.receive().await;
                        match reply {
                            Ok(historical_data_received) => match historical_data_received {
                                Ok(true) => return Ok(()),
                                Ok(false) => (),
                                Err(e) => return Err(e),
                            },
                            Err(_) => return Err(DdsError::Error(String::from("Channel error"))),
                        }
                    }
                }),
            )
            .await?
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_publication_data(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            return Err(DdsError::NotEnabled);
        }

        data_reader
            .matched_publication_list
            .iter()
            .find(|x| &x.key().value == publication_handle.as_ref())
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    #[tracing::instrument(skip(self))]
    pub fn get_matched_publications(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<Vec<InstanceHandle>> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        Ok(data_reader.get_matched_publications())
    }

    #[tracing::instrument(skip(self))]
    pub async fn set_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        qos: QosKind<DataReaderQos>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let qos = match qos {
            QosKind::Default => subscriber.default_data_reader_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };

        qos.is_consistent()?;
        if data_reader.enabled {
            data_reader.qos.check_immutability(&qos)?
        }

        data_reader.qos = qos;

        if data_reader.enabled {
            self.announce_data_reader(subscriber_handle, data_reader_handle)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn get_data_reader_qos(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<DataReaderQos> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        Ok(data_reader.qos.clone())
    }

    #[tracing::instrument(skip(self, listener_sender))]
    pub fn set_data_reader_listener(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        data_reader.listener_sender = listener_sender;
        data_reader.listener_mask = listener_mask;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn is_historical_data_received(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };

        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            return Err(DdsError::NotEnabled);
        };

        match data_reader.qos.durability.kind {
            DurabilityQosPolicyKind::Volatile => {
                return Err(DdsError::IllegalOperation);
            }
            DurabilityQosPolicyKind::TransientLocal
            | DurabilityQosPolicyKind::Transient
            | DurabilityQosPolicyKind::Persistent => (),
        };

        if let TransportReaderKind::Stateful(r) = &data_reader.transport_reader {
            Ok(r.is_historical_data_received().await)
        } else {
            Ok(true)
        }
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn enable_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) -> DdsResult<()> {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return Err(DdsError::AlreadyDeleted);
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return Err(DdsError::AlreadyDeleted);
        };
        if !data_reader.enabled {
            data_reader.enable();

            let discovered_writer_list: Vec<_> =
                self.domain_participant.discovered_writer_list.to_vec();
            for discovered_writer_data in discovered_writer_list {
                self.add_discovered_writer(
                    discovered_writer_data,
                    subscriber_handle,
                    data_reader_handle,
                    participant_address.clone(),
                )
                .await;
            }

            self.announce_data_reader(subscriber_handle, data_reader_handle)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn announce_participant(&mut self) {
        if self.domain_participant.enabled {
            let participant_builtin_topic_data = ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.transport.guid().into(),
                },
                user_data: self.domain_participant.qos.user_data.clone(),
            };
            let participant_proxy = ParticipantProxy {
                domain_id: Some(self.domain_participant.domain_id),
                domain_tag: self.domain_participant.domain_tag.clone(),
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
                discovered_participant_list: self
                    .domain_participant
                    .discovered_participant_list
                    .iter()
                    .map(|p| InstanceHandle::new(p.dds_participant_data.key().value))
                    .collect(),
            };
            let timestamp = self.get_current_time();

            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .lookup_datawriter_mut(DCPS_PARTICIPANT)
            {
                if let Ok(serialized_data) = spdp_discovered_participant_data.serialize_data() {
                    dw.write_w_timestamp(serialized_data, timestamp, &self.clock_handle)
                        .await
                        .ok();
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn announce_deleted_participant(&mut self) {
        if self.domain_participant.enabled {
            let timestamp = self.get_current_time();
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .lookup_datawriter_mut(DCPS_PARTICIPANT)
            {
                let key = InstanceHandle::new(self.transport.guid().into());
                if let Ok(serialized_data) = key.serialize_data() {
                    dw.dispose_w_timestamp(serialized_data, timestamp)
                        .await
                        .ok();
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn announce_data_writer(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher.get_data_writer(data_writer_handle) else {
            return;
        };
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == data_writer.topic_name)
        else {
            return;
        };

        let topic_data = topic.qos.topic_data.clone();

        let dds_publication_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey {
                value: data_writer.transport_writer.guid().into(),
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
            presentation: publisher.qos.presentation.clone(),
            partition: publisher.qos.partition.clone(),
            topic_data,
            group_data: publisher.qos.group_data.clone(),
            representation: data_writer.qos.representation.clone(),
        };
        let writer_proxy = WriterProxy {
            remote_writer_guid: data_writer.transport_writer.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
        };
        let discovered_writer_data = DiscoveredWriterData {
            dds_publication_data,
            writer_proxy,
        };
        let timestamp = self.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .lookup_datawriter_mut(DCPS_PUBLICATION)
        {
            if let Ok(serialized_data) = discovered_writer_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp, &self.clock_handle)
                    .await
                    .ok();
            }
        }
    }

    #[tracing::instrument(skip(self, data_writer))]
    async fn announce_deleted_data_writer(&mut self, data_writer: DataWriterEntity<R, T>) {
        let timestamp = self.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .lookup_datawriter_mut(DCPS_PUBLICATION)
        {
            let key = InstanceHandle::new(data_writer.transport_writer.guid().into());
            if let Ok(serialized_data) = key.serialize_data() {
                dw.dispose_w_timestamp(serialized_data, timestamp)
                    .await
                    .ok();
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn announce_data_reader(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return;
        };
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == data_reader.topic_name)
        else {
            return;
        };

        let guid = data_reader.transport_reader.guid();
        let dds_subscription_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey { value: guid.into() },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: data_reader.topic_name.clone(),
            type_name: data_reader.type_name.clone(),
            durability: data_reader.qos.durability.clone(),
            deadline: data_reader.qos.deadline.clone(),
            latency_budget: data_reader.qos.latency_budget.clone(),
            liveliness: data_reader.qos.liveliness.clone(),
            reliability: data_reader.qos.reliability.clone(),
            ownership: data_reader.qos.ownership.clone(),
            destination_order: data_reader.qos.destination_order.clone(),
            user_data: data_reader.qos.user_data.clone(),
            time_based_filter: data_reader.qos.time_based_filter.clone(),
            presentation: subscriber.qos.presentation.clone(),
            partition: subscriber.qos.partition.clone(),
            topic_data: topic.qos.topic_data.clone(),
            group_data: subscriber.qos.group_data.clone(),
            representation: data_reader.qos.representation.clone(),
        };
        let reader_proxy = ReaderProxy {
            remote_reader_guid: data_reader.transport_reader.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        };
        let discovered_reader_data = DiscoveredReaderData {
            dds_subscription_data,
            reader_proxy,
        };
        let timestamp = self.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .lookup_datawriter_mut(DCPS_SUBSCRIPTION)
        {
            if let Ok(serialized_data) = discovered_reader_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp, &self.clock_handle)
                    .await
                    .ok();
            }
        }
    }

    #[tracing::instrument(skip(self, data_reader))]
    async fn announce_deleted_data_reader(&mut self, data_reader: DataReaderEntity<R, T>) {
        let timestamp = self.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .lookup_datawriter_mut(DCPS_SUBSCRIPTION)
        {
            let guid = data_reader.transport_reader.guid();
            let key = InstanceHandle::new(guid.into());
            if let Ok(serialized_data) = key.serialize_data() {
                dw.dispose_w_timestamp(serialized_data, timestamp)
                    .await
                    .ok();
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn announce_topic(&mut self, topic_name: String) {
        let Some(topic) = self
            .domain_participant
            .topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
        else {
            return;
        };

        let topic_builtin_topic_data = TopicBuiltinTopicData {
            key: BuiltInTopicKey {
                value: topic.instance_handle.into(),
            },
            name: topic.topic_name.clone(),
            type_name: topic.type_name.clone(),
            durability: topic.qos.durability.clone(),
            deadline: topic.qos.deadline.clone(),
            latency_budget: topic.qos.latency_budget.clone(),
            liveliness: topic.qos.liveliness.clone(),
            reliability: topic.qos.reliability.clone(),
            transport_priority: topic.qos.transport_priority.clone(),
            lifespan: topic.qos.lifespan.clone(),
            destination_order: topic.qos.destination_order.clone(),
            history: topic.qos.history.clone(),
            resource_limits: topic.qos.resource_limits.clone(),
            ownership: topic.qos.ownership.clone(),
            topic_data: topic.qos.topic_data.clone(),
            representation: topic.qos.representation.clone(),
        };

        let timestamp = self.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .lookup_datawriter_mut(DCPS_TOPIC)
        {
            if let Ok(serialized_data) = topic_builtin_topic_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp, &self.clock_handle)
                    .await
                    .ok();
            }
        }
    }

    #[tracing::instrument(skip(self, participant_address))]
    async fn add_discovered_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
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
            .discovered_participant_list
            .iter()
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
            .any(|n| publisher.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_reader_data
            .dds_subscription_data
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                publisher
                    .qos
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_any_local_regex_matched_with_received_partition_qos = publisher
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                discovered_reader_data
                    .dds_subscription_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = discovered_reader_data.dds_subscription_data.partition
            == publisher.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let publisher_qos = publisher.qos.clone();
            let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
                return;
            };

            let is_matched_topic_name =
                discovered_reader_data.dds_subscription_data.topic_name == data_writer.topic_name;
            let is_matched_type_name = discovered_reader_data.dds_subscription_data.get_type_name()
                == data_writer.type_name;

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_reader_incompatible_qos_policy_list(
                        &data_writer.qos,
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

                    let reader_proxy = transport::types::ReaderProxy {
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
                    if let TransportWriterKind::Stateful(w) = &mut data_writer.transport_writer {
                        w.add_matched_reader(reader_proxy).await;
                    }

                    if data_writer
                        .listener_mask
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
                        if let Some(l) = &data_writer.listener_sender {
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .await
                                .ok();
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
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
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
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .await
                                .ok();
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
                    data_writer
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::PublicationMatched,
                        })
                        .await;
                } else {
                    data_writer.add_incompatible_subscription(
                        InstanceHandle::new(
                            discovered_reader_data.dds_subscription_data.key().value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_writer
                        .listener_mask
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
                        if let Some(l) = &data_writer.listener_sender {
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .await
                                .ok();
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
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
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
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .await
                                .ok();
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
                    data_writer
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::OfferedIncompatibleQos,
                        })
                        .await;
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn remove_discovered_reader(
        &mut self,
        subscription_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) {
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return;
        };
        if data_writer
            .matched_subscription_list
            .iter()
            .find(|x| subscription_handle.as_ref() == &x.key().value)
            .is_some()
        {
            data_writer.remove_matched_subscription(&subscription_handle);

            data_writer
                .status_condition
                .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                    state: StatusKind::PublicationMatched,
                })
                .await;
        }
    }

    #[tracing::instrument(skip(self, participant_address))]
    async fn add_discovered_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
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
            .discovered_participant_list
            .iter()
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
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let is_any_name_matched = discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .any(|n| subscriber.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                subscriber
                    .qos
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_any_local_regex_matched_with_received_partition_qos = subscriber
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                discovered_writer_data
                    .dds_publication_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = discovered_writer_data.dds_publication_data.partition
            == subscriber.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let subscriber_qos = subscriber.qos.clone();
            let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
                return;
            };
            let is_matched_topic_name =
                discovered_writer_data.dds_publication_data.topic_name == data_reader.topic_name;
            let is_matched_type_name = discovered_writer_data.dds_publication_data.get_type_name()
                == data_reader.type_name;

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_writer_incompatible_qos_policy_list::<R, T>(
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
                    let reliability_kind = match data_reader.qos.reliability.kind {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind = match data_reader.qos.durability.kind {
                        DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                        DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                        DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                        DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                    };
                    let writer_proxy = transport::types::WriterProxy {
                        remote_writer_guid: discovered_writer_data.writer_proxy.remote_writer_guid,
                        remote_group_entity_id: discovered_writer_data
                            .writer_proxy
                            .remote_group_entity_id,
                        unicast_locator_list,
                        multicast_locator_list,
                        reliability_kind,
                        durability_kind,
                    };
                    if let TransportReaderKind::Stateful(r) = &mut data_reader.transport_reader {
                        r.add_matched_writer(writer_proxy).await;
                    }

                    if data_reader
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .await
                                .ok();
                        }
                    } else if subscriber
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .await
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::SubscriptionMatched,
                        })
                        .await;
                } else {
                    data_reader.add_requested_incompatible_qos(
                        InstanceHandle::new(
                            discovered_writer_data.dds_publication_data.key().value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_reader
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .await
                                .ok();
                        }
                    } else if subscriber
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .await
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::RequestedIncompatibleQos,
                        })
                        .await;
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn remove_discovered_writer(
        &mut self,
        publication_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return;
        };
        if data_reader
            .matched_publication_list
            .iter()
            .any(|x| &x.key().value == publication_handle.as_ref())
        {
            data_reader
                .remove_matched_publication(&publication_handle)
                .await;
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn add_builtin_participants_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
    ) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_participant_data) =
                    SpdpDiscoveredParticipantData::deserialize_data(
                        cache_change.data_value.as_ref(),
                    )
                {
                    self.add_discovered_participant(discovered_participant_data)
                        .await;
                }
            }
            ChangeKind::NotAliveDisposed => {
                if let Ok(discovered_participant_handle) =
                    InstanceHandle::deserialize_data(cache_change.data_value.as_ref())
                {
                    self.remove_discovered_participant(discovered_participant_handle);
                }
            }
            ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (), // Do nothing,
        }

        let reception_timestamp = self.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.topic_name == DCPS_PARTICIPANT)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn add_builtin_publications_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_writer_data) =
                    DiscoveredWriterData::deserialize_data(cache_change.data_value.as_ref())
                {
                    let publication_builtin_topic_data =
                        &discovered_writer_data.dds_publication_data;
                    if self
                        .domain_participant
                        .find_topic(&publication_builtin_topic_data.topic_name)
                        .is_none()
                    {
                        let writer_topic = TopicBuiltinTopicData {
                            key: BuiltInTopicKey::default(),
                            name: publication_builtin_topic_data.topic_name.clone(),
                            type_name: publication_builtin_topic_data.type_name.clone(),
                            durability: publication_builtin_topic_data.durability().clone(),
                            deadline: publication_builtin_topic_data.deadline().clone(),
                            latency_budget: publication_builtin_topic_data.latency_budget().clone(),
                            liveliness: publication_builtin_topic_data.liveliness().clone(),
                            reliability: publication_builtin_topic_data.reliability().clone(),
                            transport_priority: TransportPriorityQosPolicy::default(),
                            lifespan: publication_builtin_topic_data.lifespan().clone(),
                            destination_order: publication_builtin_topic_data
                                .destination_order()
                                .clone(),
                            history: HistoryQosPolicy::default(),
                            resource_limits: ResourceLimitsQosPolicy::default(),
                            ownership: publication_builtin_topic_data.ownership().clone(),
                            topic_data: publication_builtin_topic_data.topic_data().clone(),
                            representation: publication_builtin_topic_data.representation().clone(),
                        };
                        self.domain_participant.add_discovered_topic(writer_topic);
                    }

                    self.domain_participant
                        .add_discovered_writer(discovered_writer_data.clone());
                    let mut handle_list = Vec::new();
                    for subscriber in &self.domain_participant.user_defined_subscriber_list {
                        for data_reader in subscriber.data_reader_list.iter() {
                            handle_list
                                .push((subscriber.instance_handle, data_reader.instance_handle));
                        }
                    }
                    for (subscriber_handle, data_reader_handle) in handle_list {
                        self.add_discovered_writer(
                            discovered_writer_data.clone(),
                            subscriber_handle,
                            data_reader_handle,
                            participant_address.clone(),
                        )
                        .await;
                    }
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                if let Ok(discovered_writer_handle) =
                    InstanceHandle::deserialize_data(cache_change.data_value.as_ref())
                {
                    self.domain_participant
                        .remove_discovered_writer(&discovered_writer_handle);

                    let mut handle_list = Vec::new();
                    for subscriber in &self.domain_participant.user_defined_subscriber_list {
                        for data_reader in subscriber.data_reader_list.iter() {
                            handle_list
                                .push((subscriber.instance_handle, data_reader.instance_handle));
                        }
                    }
                    for (subscriber_handle, data_reader_handle) in handle_list {
                        self.remove_discovered_writer(
                            discovered_writer_handle,
                            subscriber_handle,
                            data_reader_handle,
                        )
                        .await;
                    }
                }
            }
            ChangeKind::AliveFiltered | ChangeKind::NotAliveUnregistered => (),
        }

        let reception_timestamp = self.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.topic_name == DCPS_PUBLICATION)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn add_builtin_subscriptions_detector_cache_change(
        &mut self,
        cache_change: CacheChange,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_reader_data) =
                    DiscoveredReaderData::deserialize_data(cache_change.data_value.as_ref())
                {
                    if self
                        .domain_participant
                        .find_topic(&discovered_reader_data.dds_subscription_data.topic_name)
                        .is_none()
                    {
                        let reader_topic = TopicBuiltinTopicData {
                            key: BuiltInTopicKey::default(),
                            name: discovered_reader_data
                                .dds_subscription_data
                                .topic_name
                                .to_string(),
                            type_name: discovered_reader_data
                                .dds_subscription_data
                                .get_type_name()
                                .to_string(),

                            topic_data: discovered_reader_data
                                .dds_subscription_data
                                .topic_data()
                                .clone(),
                            durability: discovered_reader_data
                                .dds_subscription_data
                                .durability()
                                .clone(),
                            deadline: discovered_reader_data
                                .dds_subscription_data
                                .deadline()
                                .clone(),
                            latency_budget: discovered_reader_data
                                .dds_subscription_data
                                .latency_budget()
                                .clone(),
                            liveliness: discovered_reader_data
                                .dds_subscription_data
                                .liveliness()
                                .clone(),
                            reliability: discovered_reader_data
                                .dds_subscription_data
                                .reliability()
                                .clone(),
                            destination_order: discovered_reader_data
                                .dds_subscription_data
                                .destination_order()
                                .clone(),
                            history: HistoryQosPolicy::default(),
                            resource_limits: ResourceLimitsQosPolicy::default(),
                            transport_priority: TransportPriorityQosPolicy::default(),
                            lifespan: LifespanQosPolicy::default(),
                            ownership: discovered_reader_data
                                .dds_subscription_data
                                .ownership()
                                .clone(),
                            representation: discovered_reader_data
                                .dds_subscription_data
                                .representation()
                                .clone(),
                        };
                        self.domain_participant.add_discovered_topic(reader_topic);
                    }

                    self.domain_participant
                        .add_discovered_reader(discovered_reader_data.clone());
                    let mut handle_list = Vec::new();
                    for publisher in &self.domain_participant.user_defined_publisher_list {
                        for data_writer in publisher.data_writer_list() {
                            handle_list
                                .push((publisher.instance_handle, data_writer.instance_handle));
                        }
                    }
                    for (publisher_handle, data_writer_handle) in handle_list {
                        self.add_discovered_reader(
                            discovered_reader_data.clone(),
                            publisher_handle,
                            data_writer_handle,
                            participant_address.clone(),
                        )
                        .await;
                    }
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                if let Ok(discovered_reader_handle) =
                    InstanceHandle::deserialize_data(cache_change.data_value.as_ref())
                {
                    self.domain_participant
                        .remove_discovered_reader(&discovered_reader_handle);

                    let mut handle_list = Vec::new();
                    for publisher in &self.domain_participant.user_defined_publisher_list {
                        for data_writer in publisher.data_writer_list() {
                            handle_list
                                .push((publisher.instance_handle, data_writer.instance_handle));
                        }
                    }

                    for (publisher_handle, data_writer_handle) in handle_list {
                        self.remove_discovered_reader(
                            discovered_reader_handle,
                            publisher_handle,
                            data_writer_handle,
                        )
                        .await;
                    }
                }
            }
            ChangeKind::AliveFiltered | ChangeKind::NotAliveUnregistered => (),
        }

        let reception_timestamp = self.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.topic_name == DCPS_SUBSCRIPTION)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn add_builtin_topics_detector_cache_change(&mut self, cache_change: CacheChange) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(topic_builtin_topic_data) =
                    TopicBuiltinTopicData::deserialize_data(cache_change.data_value.as_ref())
                {
                    self.domain_participant
                        .add_discovered_topic(topic_builtin_topic_data.clone());
                    for topic in self.domain_participant.topic_list.iter_mut() {
                        if topic.topic_name == topic_builtin_topic_data.name()
                            && topic.type_name == topic_builtin_topic_data.get_type_name()
                            && !is_discovered_topic_consistent(
                                &topic.qos,
                                &topic_builtin_topic_data,
                            )
                        {
                            topic.inconsistent_topic_status.total_count += 1;
                            topic.inconsistent_topic_status.total_count_change += 1;
                            topic
                                .status_condition
                                .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                                    state: StatusKind::InconsistentTopic,
                                })
                                .await;
                        }
                    }
                }
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (),
        }

        let reception_timestamp = self.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.topic_name == DCPS_TOPIC)
        {
            reader
                .add_reader_change(cache_change, reception_timestamp)
                .ok();
        }
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn add_cache_change(
        &mut self,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
        cache_change: CacheChange,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let reception_timestamp = self.get_current_time();
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };

        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return;
        };
        let writer_instance_handle = InstanceHandle::new(cache_change.writer_guid.into());

        if data_reader
            .matched_publication_list
            .iter()
            .any(|x| &x.key().value == writer_instance_handle.as_ref())
        {
            match data_reader.add_reader_change(cache_change, reception_timestamp) {
                Ok(AddChangeResult::Added(change_instance_handle)) => {
                    if let DurationKind::Finite(deadline_missed_period) =
                        data_reader.qos.deadline.period
                    {
                        let mut timer_handle = self.timer_handle.clone();
                        let participant_address = participant_address.clone();

                        self.spawner_handle.spawn(async move {
                            loop {
                                timer_handle.delay(deadline_missed_period.into()).await;
                                participant_address
                                    .send(DcpsDomainParticipantMail::Event(
                                        EventServiceMail::RequestedDeadlineMissed {
                                            subscriber_handle,
                                            data_reader_handle,
                                            change_instance_handle,
                                            participant_address: participant_address.clone(),
                                        },
                                    ))
                                    .await
                                    .ok();
                            }
                        });
                    }
                    let deta_reader_on_data_available_active = data_reader
                        .listener_mask
                        .contains(&StatusKind::DataAvailable);

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };

                    if subscriber
                        .listener_mask
                        .contains(&StatusKind::DataOnReaders)
                    {
                        let Ok(the_subscriber) = self
                            .get_subscriber_async(participant_address.clone(), subscriber_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::DataOnReaders { the_subscriber })
                                .await
                                .ok();
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::DataAvailable { the_reader })
                                .await
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };

                    subscriber
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::DataOnReaders,
                        })
                        .await;
                    let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::DataAvailable,
                        })
                        .await;
                }
                Ok(AddChangeResult::NotAdded) => (), // Do nothing
                Ok(AddChangeResult::Rejected(instance_handle, sample_rejected_status_kind)) => {
                    data_reader.increment_sample_rejected_status(
                        instance_handle,
                        sample_rejected_status_kind,
                    );

                    if data_reader
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::SampleRejected { the_reader, status })
                                .await
                                .ok();
                        };
                    } else if subscriber
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_sample_rejected_status();
                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::SampleRejected { status, the_reader })
                                .await
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
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
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };

                        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_sample_rejected_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::SampleRejected { status, the_reader })
                                .await
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };

                    let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                            state: StatusKind::SampleRejected,
                        })
                        .await;
                }
                Err(_) => (),
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn remove_writer_change(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        sequence_number: i64,
    ) {
        if let Some(p) = self.domain_participant.get_mut_publisher(publisher_handle) {
            if let Some(dw) = p.get_mut_data_writer(data_writer_handle) {
                dw.transport_writer
                    .history_cache()
                    .remove_change(sequence_number)
                    .await;
            }
        }
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn offered_deadline_missed(
        &mut self,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) {
        let current_time = self.get_current_time();
        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return;
        };

        if let DurationKind::Finite(deadline) = data_writer.qos.deadline.period {
            match data_writer.get_instance_write_time(change_instance_handle) {
                Some(t) => {
                    if current_time - t < deadline {
                        return;
                    }
                }
                None => return,
            }
        } else {
            return;
        }

        data_writer
            .offered_deadline_missed_status
            .last_instance_handle = change_instance_handle;
        data_writer.offered_deadline_missed_status.total_count += 1;
        data_writer
            .offered_deadline_missed_status
            .total_count_change += 1;

        if data_writer
            .listener_mask
            .contains(&StatusKind::OfferedDeadlineMissed)
        {
            let status = data_writer.get_offered_deadline_missed_status().await;
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

            if let Some(l) = &data_writer.listener_sender {
                l.send(ListenerMail::OfferedDeadlineMissed { the_writer, status })
                    .await
                    .ok();
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
            let status = data_writer.get_offered_deadline_missed_status().await;
            if let Some(l) = publisher.listener() {
                l.send(ListenerMail::OfferedDeadlineMissed { the_writer, status })
                    .await
                    .ok();
            }
        } else if self
            .domain_participant
            .listener_mask
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
            let status = data_writer.get_offered_deadline_missed_status().await;
            if let Some(l) = &self.domain_participant.listener_sender {
                l.send(ListenerMail::OfferedDeadlineMissed { the_writer, status })
                    .await
                    .ok();
            }
        }

        let Some(publisher) = self.domain_participant.get_mut_publisher(publisher_handle) else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(data_writer_handle) else {
            return;
        };
        data_writer
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                state: StatusKind::OfferedDeadlineMissed,
            })
            .await;
    }

    #[tracing::instrument(skip(self, participant_address))]
    pub async fn requested_deadline_missed(
        &mut self,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
        change_instance_handle: InstanceHandle,
        participant_address: R::ChannelSender<DcpsDomainParticipantMail<R>>,
    ) {
        let current_time = self.get_current_time();
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return;
        };

        if let DurationKind::Finite(deadline) = data_reader.qos.deadline.period {
            if let Some(t) = data_reader.get_instance_received_time(&change_instance_handle) {
                if current_time - t < deadline {
                    return;
                }
            } else {
                return;
            }
        }

        data_reader.remove_instance_ownership(&change_instance_handle);
        data_reader.increment_requested_deadline_missed_status(change_instance_handle);

        if data_reader
            .listener_mask
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
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
                return;
            };
            if let Some(l) = &data_reader.listener_sender {
                l.send(ListenerMail::RequestedDeadlineMissed { the_reader, status })
                    .await
                    .ok();
            }
        } else if subscriber
            .listener_mask
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
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
                return;
            };
            let status = data_reader.get_requested_deadline_missed_status();
            if let Some(l) = &subscriber.listener_sender {
                l.send(ListenerMail::RequestedDeadlineMissed { status, the_reader })
                    .await
                    .ok();
            }
        } else if self
            .domain_participant
            .listener_mask
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
                .user_defined_subscriber_list
                .iter_mut()
                .find(|x| x.instance_handle == subscriber_handle)
            else {
                return;
            };
            let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
                return;
            };
            let status = data_reader.get_requested_deadline_missed_status();
            if let Some(l) = &self.domain_participant.listener_sender {
                l.send(ListenerMail::RequestedDeadlineMissed { status, the_reader })
                    .await
                    .ok();
            }
        }
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(data_reader_handle) else {
            return;
        };

        data_reader
            .status_condition
            .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                state: StatusKind::RequestedDeadlineMissed,
            })
            .await;
    }

    #[tracing::instrument(skip(self))]
    async fn add_discovered_participant(
        &mut self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // IN CASE no domain id was transmitted the a local domain id is assumed
        // (as specified in Table 9.19 - ParameterId mapping and default values)
        let is_domain_id_matching = match discovered_participant_data.participant_proxy.domain_id {
            Some(id) => id == self.domain_participant.domain_id,
            None => true,
        };
        let is_domain_tag_matching = discovered_participant_data.participant_proxy.domain_tag
            == self.domain_participant.domain_tag;

        let is_participant_discovered = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .any(|p| {
                p.dds_participant_data.key().value
                    == discovered_participant_data.dds_participant_data.key.value
            });

        if is_domain_id_matching && is_domain_tag_matching && !is_participant_discovered {
            self.add_matched_publications_detector(&discovered_participant_data)
                .await;
            self.add_matched_publications_announcer(&discovered_participant_data)
                .await;
            self.add_matched_subscriptions_detector(&discovered_participant_data)
                .await;
            self.add_matched_subscriptions_announcer(&discovered_participant_data)
                .await;
            self.add_matched_topics_detector(&discovered_participant_data)
                .await;
            self.add_matched_topics_announcer(&discovered_participant_data)
                .await;

            self.announce_participant().await;
        }

        self.domain_participant
            .add_discovered_participant(discovered_participant_data);
    }

    #[tracing::instrument(skip(self))]
    fn remove_discovered_participant(&mut self, discovered_participant: InstanceHandle) {
        self.domain_participant
            .discovered_participant_list
            .retain(|p| &p.dds_participant_data.key().value != discovered_participant.as_ref());
    }

    #[tracing::instrument(skip(self))]
    async fn add_matched_publications_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = transport::types::ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy).await,
                    TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn add_matched_publications_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;

            let writer_proxy = transport::types::WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
            };
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    TransportReaderKind::Stateful(r) => r.add_matched_writer(writer_proxy).await,
                    TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn add_matched_subscriptions_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = transport::types::ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy).await,
                    TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn add_matched_subscriptions_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;

            let writer_proxy = transport::types::WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
            };
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    TransportReaderKind::Stateful(r) => r.add_matched_writer(writer_proxy).await,
                    TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn add_matched_topics_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = transport::types::ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy).await,
                    TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn add_matched_topics_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;

            let writer_proxy = transport::types::WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::TransientLocal,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
            };
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    TransportReaderKind::Stateful(r) => r.add_matched_writer(writer_proxy).await,
                    TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }
}

#[tracing::instrument(skip(type_support))]
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

#[tracing::instrument]
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

#[tracing::instrument(skip(data_reader))]
fn get_discovered_writer_incompatible_qos_policy_list<
    R: DdsRuntime,
    T: TransportParticipantFactory,
>(
    data_reader: &DataReaderEntity<R, T>,
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
    if &data_reader.qos.durability > publication_builtin_topic_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos.deadline < publication_builtin_topic_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &data_reader.qos.latency_budget > publication_builtin_topic_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &data_reader.qos.liveliness > publication_builtin_topic_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if data_reader.qos.reliability.kind > publication_builtin_topic_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos.destination_order > publication_builtin_topic_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if data_reader.qos.ownership.kind != publication_builtin_topic_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = publication_builtin_topic_data
        .representation()
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !data_reader
        .qos
        .representation
        .value
        .contains(writer_offered_representation)
    {
        // Empty list is interpreted as containing XCDR_DATA_REPRESENTATION
        if !(writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && data_reader.qos.representation.value.is_empty())
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

fn fnmatch_to_regex(pattern: &str) -> String {
    fn flush_literal(out: &mut String, lit: &mut String) {
        if !lit.is_empty() {
            out.push_str(&regex::escape(lit));
            lit.clear();
        }
    }

    let mut out = String::from("^");
    let mut literal = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // backslash escapes next char literally
            '\\' => {
                if let Some(next) = chars.next() {
                    literal.push(next);
                } else {
                    literal.push('\\');
                }
            }

            // glob wildcards
            '*' => {
                flush_literal(&mut out, &mut literal);
                out.push_str(".*");
            }
            '?' => {
                flush_literal(&mut out, &mut literal);
                out.push('.');
            }

            // character class
            '[' => {
                flush_literal(&mut out, &mut literal);

                let mut class = String::from("[");
                // handle fnmatch negation [!...] -> regex [^...]
                if let Some(&next) = chars.peek() {
                    if next == '!' {
                        chars.next();
                        class.push('^');
                    } else if next == '^' {
                        // treat ^ the same if user used it
                        chars.next();
                        class.push('^');
                    }
                }

                let mut closed = false;
                while let Some(ch) = chars.next() {
                    class.push(ch);
                    if ch == ']' {
                        closed = true;
                        break;
                    }
                    // preserve escaped chars inside class
                    if ch == '\\' {
                        if let Some(esc) = chars.next() {
                            class.push(esc);
                        }
                    }
                }

                if closed {
                    out.push_str(&class);
                } else {
                    // unclosed '[' — treat as literal
                    literal.push('[');
                    literal.push_str(&class[1..]); // append rest as literal
                }
            }

            '+' => {
                flush_literal(&mut out, &mut literal);
                out.push('+'); // regex plus (quantifier)
            }

            // default: accumulate literal characters (will be escaped when flushed)
            other => literal.push(other),
        }
    }

    flush_literal(&mut out, &mut literal);
    out.push('$');
    out
}

pub const BUILT_IN_TOPIC_NAME_LIST: [&str; 4] = [
    DCPS_PARTICIPANT,
    DCPS_TOPIC,
    DCPS_PUBLICATION,
    DCPS_SUBSCRIPTION,
];

pub struct DomainParticipantEntity<R: DdsRuntime, T: TransportParticipantFactory> {
    domain_id: DomainId,
    domain_tag: String,
    instance_handle: InstanceHandle,
    qos: DomainParticipantQos,
    builtin_subscriber: SubscriberEntity<R, T>,
    builtin_publisher: PublisherEntity<R, T>,
    user_defined_subscriber_list: Vec<SubscriberEntity<R, T>>,
    default_subscriber_qos: SubscriberQos,
    user_defined_publisher_list: Vec<PublisherEntity<R, T>>,
    default_publisher_qos: PublisherQos,
    topic_list: Vec<TopicEntity<R>>,
    content_filtered_topic_list: Vec<ContentFilteredTopicEntity>,
    default_topic_qos: TopicQos,
    discovered_participant_list: Vec<SpdpDiscoveredParticipantData>,
    discovered_topic_list: Vec<TopicBuiltinTopicData>,
    discovered_reader_list: Vec<DiscoveredReaderData>,
    discovered_writer_list: Vec<DiscoveredWriterData>,
    enabled: bool,
    ignored_participants: Vec<InstanceHandle>,
    ignored_publications: Vec<InstanceHandle>,
    ignored_subcriptions: Vec<InstanceHandle>,
    _ignored_topic_list: Vec<InstanceHandle>,
    listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
    listener_mask: Vec<StatusKind>,
}

impl<R: DdsRuntime, T: TransportParticipantFactory> DomainParticipantEntity<R, T> {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        domain_id: DomainId,
        domain_participant_qos: DomainParticipantQos,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
        instance_handle: InstanceHandle,
        builtin_publisher: PublisherEntity<R, T>,
        builtin_subscriber: SubscriberEntity<R, T>,
        topic_list: Vec<TopicEntity<R>>,
        domain_tag: String,
    ) -> Self {
        Self {
            domain_id,
            instance_handle,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: Vec::new(),
            default_subscriber_qos: SubscriberQos::const_default(),
            user_defined_publisher_list: Vec::new(),
            default_publisher_qos: PublisherQos::const_default(),
            topic_list,
            content_filtered_topic_list: Vec::new(),
            default_topic_qos: TopicQos::const_default(),
            discovered_participant_list: Vec::new(),
            discovered_topic_list: Vec::new(),
            discovered_reader_list: Vec::new(),
            discovered_writer_list: Vec::new(),
            enabled: false,
            ignored_participants: Vec::new(),
            ignored_publications: Vec::new(),
            ignored_subcriptions: Vec::new(),
            _ignored_topic_list: Vec::new(),
            listener_sender,
            listener_mask,
            domain_tag,
        }
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn builtin_subscriber(&self) -> &SubscriberEntity<R, T> {
        &self.builtin_subscriber
    }

    pub fn add_discovered_topic(&mut self, topic_builtin_topic_data: TopicBuiltinTopicData) {
        match self
            .discovered_topic_list
            .iter_mut()
            .find(|t| t.key() == topic_builtin_topic_data.key())
        {
            Some(x) => *x = topic_builtin_topic_data,
            None => self.discovered_topic_list.push(topic_builtin_topic_data),
        }
    }

    pub fn remove_discovered_writer(&mut self, discovered_writer_handle: &InstanceHandle) {
        self.discovered_writer_list
            .retain(|x| &x.dds_publication_data.key().value != discovered_writer_handle.as_ref());
    }

    pub fn get_discovered_topic_data(
        &self,
        topic_handle: &InstanceHandle,
    ) -> Option<&TopicBuiltinTopicData> {
        self.discovered_topic_list
            .iter()
            .find(|x| &x.key().value == topic_handle.as_ref())
    }

    pub fn find_topic(&self, topic_name: &str) -> Option<&TopicBuiltinTopicData> {
        self.discovered_topic_list
            .iter()
            .find(|&discovered_topic_data| discovered_topic_data.name() == topic_name)
    }

    pub fn add_discovered_participant(
        &mut self,
        discovered_participant_data: SpdpDiscoveredParticipantData,
    ) {
        match self.discovered_participant_list.iter_mut().find(|p| {
            p.dds_participant_data.key() == discovered_participant_data.dds_participant_data.key()
        }) {
            Some(x) => *x = discovered_participant_data,
            None => self
                .discovered_participant_list
                .push(discovered_participant_data),
        }
    }

    pub fn add_discovered_reader(&mut self, discovered_reader_data: DiscoveredReaderData) {
        match self.discovered_reader_list.iter_mut().find(|x| {
            x.dds_subscription_data.key() == discovered_reader_data.dds_subscription_data.key()
        }) {
            Some(x) => *x = discovered_reader_data,
            None => self.discovered_reader_list.push(discovered_reader_data),
        }
    }

    pub fn remove_discovered_reader(&mut self, discovered_reader_handle: &InstanceHandle) {
        self.discovered_reader_list
            .retain(|x| &x.dds_subscription_data.key().value != discovered_reader_handle.as_ref());
    }

    pub fn add_discovered_writer(&mut self, discovered_writer_data: DiscoveredWriterData) {
        match self.discovered_writer_list.iter_mut().find(|x| {
            x.dds_publication_data.key() == discovered_writer_data.dds_publication_data.key()
        }) {
            Some(x) => *x = discovered_writer_data,
            None => self.discovered_writer_list.push(discovered_writer_data),
        }
    }

    pub fn remove_subscriber(&mut self, handle: &InstanceHandle) -> Option<SubscriberEntity<R, T>> {
        let i = self
            .user_defined_subscriber_list
            .iter()
            .position(|x| &x.instance_handle == handle)?;

        Some(self.user_defined_subscriber_list.remove(i))
    }

    pub fn get_mut_publisher(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<&mut PublisherEntity<R, T>> {
        self.user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == handle)
    }

    pub fn remove_publisher(&mut self, handle: &InstanceHandle) -> Option<PublisherEntity<R, T>> {
        let i = self
            .user_defined_publisher_list
            .iter()
            .position(|x| &x.instance_handle == handle)?;

        Some(self.user_defined_publisher_list.remove(i))
    }

    pub fn is_empty(&self) -> bool {
        let no_user_defined_topics = self
            .topic_list
            .iter()
            .filter(|t| !BUILT_IN_TOPIC_NAME_LIST.contains(&t.topic_name.as_ref()))
            .count()
            == 0;

        self.user_defined_publisher_list.is_empty()
            && self.user_defined_subscriber_list.is_empty()
            && no_user_defined_topics
    }
}

pub struct ContentFilteredTopicEntity {
    _name: String,
    related_topic_name: String,
    _filter_expression: String,
    _expression_parameters: Vec<String>,
}

impl ContentFilteredTopicEntity {
    pub fn new(
        name: String,
        related_topic_name: String,
        filter_expression: String,
        expression_parameters: Vec<String>,
    ) -> Self {
        Self {
            _name: name,
            related_topic_name,
            _filter_expression: filter_expression,
            _expression_parameters: expression_parameters,
        }
    }
}

pub struct SubscriberEntity<R: DdsRuntime, T: TransportParticipantFactory> {
    instance_handle: InstanceHandle,
    qos: SubscriberQos,
    data_reader_list: Vec<DataReaderEntity<R, T>>,
    enabled: bool,
    default_data_reader_qos: DataReaderQos,
    status_condition: Actor<R, DcpsStatusCondition<R>>,
    listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
    listener_mask: Vec<StatusKind>,
}

impl<R: DdsRuntime, T: TransportParticipantFactory> SubscriberEntity<R, T> {
    pub const fn new(
        instance_handle: InstanceHandle,
        qos: SubscriberQos,
        data_reader_list: Vec<DataReaderEntity<R, T>>,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            instance_handle,
            qos,
            data_reader_list,
            enabled: false,
            default_data_reader_qos: DataReaderQos::const_default(),
            status_condition,
            listener_sender,
            listener_mask,
        }
    }

    pub fn status_condition(&self) -> &Actor<R, DcpsStatusCondition<R>> {
        &self.status_condition
    }

    pub fn get_mut_data_reader(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<&mut DataReaderEntity<R, T>> {
        self.data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == handle)
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }
}

pub struct TopicEntity<R: DdsRuntime> {
    qos: TopicQos,
    type_name: String,
    topic_name: String,
    instance_handle: InstanceHandle,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: Actor<R, DcpsStatusCondition<R>>,
    _listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
    _status_kind: Vec<StatusKind>,
    type_support: Arc<dyn DynamicType + Send + Sync>,
}

impl<R: DdsRuntime> TopicEntity<R> {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        qos: TopicQos,
        type_name: String,
        topic_name: String,
        instance_handle: InstanceHandle,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        status_kind: Vec<StatusKind>,
        type_support: Arc<dyn DynamicType + Send + Sync>,
    ) -> Self {
        Self {
            qos,
            type_name,
            topic_name,
            instance_handle,
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::const_default(),
            status_condition,
            _listener_sender: listener_sender,
            _status_kind: status_kind,
            type_support,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub async fn get_inconsistent_topic_status(&mut self) -> InconsistentTopicStatus {
        let status = self.inconsistent_topic_status.clone();
        self.inconsistent_topic_status.total_count_change = 0;
        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::InconsistentTopic,
            })
            .await;
        status
    }
}

pub struct PublisherEntity<R: DdsRuntime, T: TransportParticipantFactory> {
    qos: PublisherQos,
    instance_handle: InstanceHandle,
    data_writer_list: Vec<DataWriterEntity<R, T>>,
    enabled: bool,
    default_datawriter_qos: DataWriterQos,
    listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
    listener_mask: Vec<StatusKind>,
}

impl<R: DdsRuntime, T: TransportParticipantFactory> PublisherEntity<R, T> {
    pub const fn new(
        qos: PublisherQos,
        instance_handle: InstanceHandle,
        data_writer_list: Vec<DataWriterEntity<R, T>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            qos,
            instance_handle,
            data_writer_list,
            enabled: false,
            default_datawriter_qos: DataWriterQos::const_default(),
            listener_sender,
            listener_mask,
        }
    }

    pub fn data_writer_list(&self) -> impl Iterator<Item = &DataWriterEntity<R, T>> {
        self.data_writer_list.iter()
    }

    pub fn drain_data_writer_list(&mut self) -> impl Iterator<Item = DataWriterEntity<R, T>> + '_ {
        self.data_writer_list.drain(..)
    }

    pub fn insert_data_writer(&mut self, data_writer: DataWriterEntity<R, T>) {
        self.data_writer_list.push(data_writer);
    }

    pub fn remove_data_writer(&mut self, handle: InstanceHandle) -> Option<DataWriterEntity<R, T>> {
        let index = self
            .data_writer_list
            .iter()
            .position(|x| x.instance_handle == handle)?;
        Some(self.data_writer_list.remove(index))
    }

    pub fn get_data_writer(&self, handle: InstanceHandle) -> Option<&DataWriterEntity<R, T>> {
        self.data_writer_list
            .iter()
            .find(|x| x.instance_handle == handle)
    }

    pub fn get_mut_data_writer(
        &mut self,
        handle: InstanceHandle,
    ) -> Option<&mut DataWriterEntity<R, T>> {
        self.data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == handle)
    }

    pub fn lookup_datawriter_mut(
        &mut self,
        topic_name: &str,
    ) -> Option<&mut DataWriterEntity<R, T>> {
        self.data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == topic_name)
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn default_datawriter_qos(&self) -> &DataWriterQos {
        &self.default_datawriter_qos
    }

    pub fn set_default_datawriter_qos(
        &mut self,
        default_datawriter_qos: DataWriterQos,
    ) -> DdsResult<()> {
        default_datawriter_qos.is_consistent()?;
        self.default_datawriter_qos = default_datawriter_qos;
        Ok(())
    }

    pub fn set_qos(&mut self, qos: PublisherQos) -> DdsResult<()> {
        self.qos = qos;
        Ok(())
    }

    pub fn set_listener(
        &mut self,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        mask: Vec<StatusKind>,
    ) {
        self.listener_sender = listener_sender;
        self.listener_mask = mask;
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }

    pub fn listener(&self) -> &Option<R::ChannelSender<ListenerMail<R>>> {
        &self.listener_sender
    }
}

pub enum TransportWriterKind<T: TransportParticipantFactory> {
    Stateful(<T::TransportParticipant as TransportParticipant>::StatefulWriter),
    Stateless(<T::TransportParticipant as TransportParticipant>::StatelessWriter),
}

impl<T: TransportParticipantFactory> TransportWriterKind<T> {
    pub fn guid(&self) -> Guid {
        match self {
            TransportWriterKind::Stateful(w) => w.guid(),
            TransportWriterKind::Stateless(w) => w.guid(),
        }
    }

    pub fn history_cache(&mut self) -> &mut dyn HistoryCache {
        match self {
            TransportWriterKind::Stateful(w) => w.history_cache(),
            TransportWriterKind::Stateless(w) => w.history_cache(),
        }
    }
}

pub struct InstancePublicationTime {
    instance: InstanceHandle,
    last_write_time: Time,
}

pub struct InstanceSamples {
    instance: InstanceHandle,
    samples: VecDeque<i64>,
}

pub struct DataWriterEntity<R: DdsRuntime, T: TransportParticipantFactory> {
    instance_handle: InstanceHandle,
    transport_writer: TransportWriterKind<T>,
    topic_name: String,
    type_name: String,
    type_support: Arc<dyn DynamicType + Send + Sync>,
    matched_subscription_list: Vec<SubscriptionBuiltinTopicData>,
    publication_matched_status: PublicationMatchedStatus,
    incompatible_subscription_list: Vec<InstanceHandle>,
    offered_incompatible_qos_status: OfferedIncompatibleQosStatus,
    enabled: bool,
    status_condition: Actor<R, DcpsStatusCondition<R>>,
    listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
    listener_mask: Vec<StatusKind>,
    max_seq_num: Option<i64>,
    last_change_sequence_number: i64,
    qos: DataWriterQos,
    registered_instance_list: Vec<InstanceHandle>,
    offered_deadline_missed_status: OfferedDeadlineMissedStatus,
    instance_publication_time: Vec<InstancePublicationTime>,
    instance_samples: Vec<InstanceSamples>,
}

impl<R: DdsRuntime, T: TransportParticipantFactory> DataWriterEntity<R, T> {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        instance_handle: InstanceHandle,
        transport_writer: TransportWriterKind<T>,
        topic_name: String,
        type_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            instance_handle,
            transport_writer,
            topic_name,
            type_name,
            type_support,
            matched_subscription_list: Vec::new(),
            publication_matched_status: PublicationMatchedStatus::const_default(),
            incompatible_subscription_list: Vec::new(),
            offered_incompatible_qos_status: OfferedIncompatibleQosStatus::const_default(),
            enabled: false,
            status_condition,
            listener_sender,
            listener_mask,
            max_seq_num: None,
            last_change_sequence_number: 0,
            qos,
            registered_instance_list: Vec::new(),
            offered_deadline_missed_status: OfferedDeadlineMissedStatus::const_default(),
            instance_publication_time: Vec::new(),
            instance_samples: Vec::new(),
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub async fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        timestamp: Time,
        clock: &impl Clock,
    ) -> DdsResult<i64> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.last_change_sequence_number += 1;

        let instance_handle =
            get_instance_handle_from_serialized_foo(&serialized_data, self.type_support.as_ref())?;

        if !self.registered_instance_list.contains(&instance_handle) {
            if self.registered_instance_list.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_list.push(instance_handle);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_instances) = self.qos.resource_limits.max_instances {
            if !self
                .instance_samples
                .iter()
                .any(|x| x.instance == instance_handle)
                && self.instance_samples.len() == max_instances as usize
            {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_samples_per_instance) =
            self.qos.resource_limits.max_samples_per_instance
        {
            // If the history Qos guarantess that the number of samples
            // is below the limit there is no need to check
            match self.qos.history.kind {
                HistoryQosPolicyKind::KeepLast(depth) if depth <= max_samples_per_instance => {}
                _ => {
                    if let Some(s) = self
                        .instance_samples
                        .iter()
                        .find(|x| x.instance == instance_handle)
                    {
                        // Only Alive changes count towards the resource limits
                        if s.samples.len() >= max_samples_per_instance as usize {
                            return Err(DdsError::OutOfResources);
                        }
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = self.qos.resource_limits.max_samples {
            let total_samples = self
                .instance_samples
                .iter()
                .fold(0, |acc, x| acc + x.samples.len());

            if total_samples >= max_samples as usize {
                return Err(DdsError::OutOfResources);
            }
        }

        let change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_data.into(),
        };
        if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
            if let Some(s) = self
                .instance_samples
                .iter_mut()
                .find(|x| x.instance == instance_handle)
            {
                if s.samples.len() == depth as usize {
                    if let Some(&smallest_seq_num_instance) = s.samples.front() {
                        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
                            let start_time = clock.now();
                            while let TransportWriterKind::Stateful(w) = &self.transport_writer {
                                if w.is_change_acknowledged(smallest_seq_num_instance).await {
                                    break;
                                }

                                if let DurationKind::Finite(t) =
                                    self.qos.reliability.max_blocking_time
                                {
                                    if (clock.now() - start_time) > t {
                                        return Err(DdsError::Timeout);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(smallest_seq_num_instance) = s.samples.pop_front() {
                        self.transport_writer
                            .history_cache()
                            .remove_change(smallest_seq_num_instance)
                            .await;
                    }
                }
            }
        }

        let seq_num = change.sequence_number;

        if seq_num > self.max_seq_num.unwrap_or(0) {
            self.max_seq_num = Some(seq_num)
        }

        match self
            .instance_publication_time
            .iter_mut()
            .find(|x| x.instance == instance_handle)
        {
            Some(x) => {
                if x.last_write_time < timestamp {
                    x.last_write_time = timestamp;
                }
            }
            None => self
                .instance_publication_time
                .push(InstancePublicationTime {
                    instance: instance_handle,
                    last_write_time: timestamp,
                }),
        }

        match self
            .instance_samples
            .iter_mut()
            .find(|x| x.instance == instance_handle)
        {
            Some(s) => s.samples.push_back(change.sequence_number),
            None => {
                let s = InstanceSamples {
                    instance: instance_handle,
                    samples: VecDeque::from([change.sequence_number]),
                };
                self.instance_samples.push(s);
            }
        }
        self.transport_writer
            .history_cache()
            .add_change(change)
            .await;
        Ok(self.last_change_sequence_number)
    }

    pub async fn dispose_w_timestamp(
        &mut self,
        serialized_key: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let has_key = {
            let mut has_key = false;
            for index in 0..self.type_support.get_member_count() {
                if self
                    .type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };
        if !has_key {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle =
            get_instance_handle_from_serialized_key(&serialized_key, self.type_support.as_ref())?;
        if !self.registered_instance_list.contains(&instance_handle) {
            return Err(DdsError::BadParameter);
        }

        if let Some(i) = self
            .instance_publication_time
            .iter()
            .position(|x| x.instance == instance_handle)
        {
            self.instance_publication_time.remove(i);
        }

        self.last_change_sequence_number += 1;

        let cache_change = CacheChange {
            kind: ChangeKind::NotAliveDisposed,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.transport_writer
            .history_cache()
            .add_change(cache_change)
            .await;

        Ok(())
    }

    pub async fn unregister_w_timestamp(
        &mut self,
        serialized_key: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let has_key = {
            let mut has_key = false;
            for index in 0..self.type_support.get_member_count() {
                if self
                    .type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };
        if !has_key {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle =
            get_instance_handle_from_serialized_key(&serialized_key, self.type_support.as_ref())?;
        if !self.registered_instance_list.contains(&instance_handle) {
            return Err(DdsError::BadParameter);
        }

        if let Some(i) = self
            .instance_publication_time
            .iter()
            .position(|x| x.instance == instance_handle)
        {
            self.instance_publication_time.remove(i);
        }

        self.last_change_sequence_number += 1;

        let cache_change = CacheChange {
            kind: ChangeKind::NotAliveDisposed,
            writer_guid: self.transport_writer.guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.transport_writer
            .history_cache()
            .add_change(cache_change)
            .await;
        Ok(())
    }

    pub fn add_matched_subscription(
        &mut self,
        subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    ) {
        match self
            .matched_subscription_list
            .iter_mut()
            .find(|x| x.key() == subscription_builtin_topic_data.key())
        {
            Some(x) => *x = subscription_builtin_topic_data,
            None => self
                .matched_subscription_list
                .push(subscription_builtin_topic_data),
        };
        self.publication_matched_status.current_count = self.matched_subscription_list.len() as i32;
        self.publication_matched_status.current_count_change += 1;
        self.publication_matched_status.total_count += 1;
        self.publication_matched_status.total_count_change += 1;
    }

    pub fn remove_matched_subscription(&mut self, subscription_handle: &InstanceHandle) {
        let Some(i) = self
            .matched_subscription_list
            .iter()
            .position(|x| &x.key().value == subscription_handle.as_ref())
        else {
            return;
        };
        self.matched_subscription_list.remove(i);
        self.publication_matched_status.current_count = self.matched_subscription_list.len() as i32;
        self.publication_matched_status.current_count_change -= 1;
    }

    pub fn add_incompatible_subscription(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        if !self.incompatible_subscription_list.contains(&handle) {
            self.offered_incompatible_qos_status.total_count += 1;
            self.offered_incompatible_qos_status.total_count_change += 1;
            self.offered_incompatible_qos_status.last_policy_id = incompatible_qos_policy_list[0];

            self.incompatible_subscription_list.push(handle);
            for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
                if let Some(policy_count) = self
                    .offered_incompatible_qos_status
                    .policies
                    .iter_mut()
                    .find(|x| x.policy_id == incompatible_qos_policy)
                {
                    policy_count.count += 1;
                } else {
                    self.offered_incompatible_qos_status
                        .policies
                        .push(QosPolicyCount {
                            policy_id: incompatible_qos_policy,
                            count: 1,
                        })
                }
            }
        }
    }

    pub fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        let status = self.offered_incompatible_qos_status.clone();
        self.offered_incompatible_qos_status.total_count_change = 0;
        status
    }

    pub fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        let status = self.publication_matched_status.clone();
        self.publication_matched_status.current_count_change = 0;
        self.publication_matched_status.total_count_change = 0;

        status
    }

    pub fn get_instance_write_time(&self, instance_handle: InstanceHandle) -> Option<Time> {
        self.instance_publication_time
            .iter()
            .find(|x| x.instance == instance_handle)
            .map(|x| x.last_write_time)
    }

    pub async fn are_all_changes_acknowledged(&self) -> bool {
        match &self.transport_writer {
            TransportWriterKind::Stateful(w) => {
                w.is_change_acknowledged(self.last_change_sequence_number)
                    .await
            }
            TransportWriterKind::Stateless(_) => true,
        }
    }

    pub async fn get_offered_deadline_missed_status(&mut self) -> OfferedDeadlineMissedStatus {
        let status = self.offered_deadline_missed_status.clone();
        self.offered_deadline_missed_status.total_count_change = 0;
        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::OfferedDeadlineMissed,
            })
            .await;

        status
    }
}

type SampleList = Vec<(Option<Arc<[u8]>>, SampleInfo)>;

pub enum AddChangeResult {
    Added(InstanceHandle),
    NotAdded,
    Rejected(InstanceHandle, SampleRejectedStatusKind),
}

struct InstanceState {
    handle: InstanceHandle,
    view_state: ViewStateKind,
    instance_state: InstanceStateKind,
    most_recent_disposed_generation_count: i32,
    most_recent_no_writers_generation_count: i32,
}

impl InstanceState {
    fn new(handle: InstanceHandle) -> Self {
        Self {
            handle,
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
        }
    }

    fn update_state(&mut self, change_kind: ChangeKind) {
        match self.instance_state {
            InstanceStateKind::Alive => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveDisposedUnregistered
                {
                    self.instance_state = InstanceStateKind::NotAliveDisposed;
                } else if change_kind == ChangeKind::NotAliveUnregistered {
                    self.instance_state = InstanceStateKind::NotAliveNoWriters;
                }
            }
            InstanceStateKind::NotAliveDisposed => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_disposed_generation_count += 1;
                }
            }
            InstanceStateKind::NotAliveNoWriters => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_no_writers_generation_count += 1;
                }
            }
        }

        match self.view_state {
            ViewStateKind::New => (),
            ViewStateKind::NotNew => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveUnregistered
                {
                    self.view_state = ViewStateKind::New;
                }
            }
        }
    }

    fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }

    fn handle(&self) -> InstanceHandle {
        self.handle
    }
}

#[derive(Debug)]
pub struct ReaderSample {
    pub kind: ChangeKind,
    pub writer_guid: [u8; 16],
    pub instance_handle: InstanceHandle,
    pub source_timestamp: Option<Time>,
    pub data_value: Arc<[u8]>,
    pub sample_state: SampleStateKind,
    pub disposed_generation_count: i32,
    pub no_writers_generation_count: i32,
    pub reception_timestamp: Time,
}

pub struct IndexedSample {
    pub index: usize,
    pub sample: (Option<Arc<[u8]>>, SampleInfo),
}

pub enum TransportReaderKind<T: TransportParticipantFactory> {
    Stateful(<T::TransportParticipant as TransportParticipant>::StatefulReader),
    Stateless(<T::TransportParticipant as TransportParticipant>::StatelessReader),
}

impl<T: TransportParticipantFactory> TransportReaderKind<T> {
    pub fn guid(&self) -> Guid {
        match self {
            TransportReaderKind::Stateful(r) => r.guid(),
            TransportReaderKind::Stateless(r) => r.guid(),
        }
    }
}

struct InstanceOwnership {
    instance_handle: InstanceHandle,
    owner_handle: [u8; 16],
    last_received_time: Time,
}

pub struct DataReaderEntity<R: DdsRuntime, T: TransportParticipantFactory> {
    instance_handle: InstanceHandle,
    sample_list: Vec<ReaderSample>,
    qos: DataReaderQos,
    topic_name: String,
    type_name: String,
    type_support: Arc<dyn DynamicType + Send + Sync>,
    requested_deadline_missed_status: RequestedDeadlineMissedStatus,
    requested_incompatible_qos_status: RequestedIncompatibleQosStatus,
    sample_rejected_status: SampleRejectedStatus,
    subscription_matched_status: SubscriptionMatchedStatus,
    matched_publication_list: Vec<PublicationBuiltinTopicData>,
    enabled: bool,
    data_available_status_changed_flag: bool,
    incompatible_writer_list: Vec<InstanceHandle>,
    status_condition: Actor<R, DcpsStatusCondition<R>>,
    listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
    listener_mask: Vec<StatusKind>,
    instances: Vec<InstanceState>,
    instance_ownership: Vec<InstanceOwnership>,
    transport_reader: TransportReaderKind<T>,
}

impl<R: DdsRuntime, T: TransportParticipantFactory> DataReaderEntity<R, T> {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        type_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        status_condition: Actor<R, DcpsStatusCondition<R>>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        listener_mask: Vec<StatusKind>,
        transport_reader: TransportReaderKind<T>,
    ) -> Self {
        Self {
            instance_handle,
            sample_list: Vec::new(),
            qos,
            topic_name,
            type_name,
            type_support,
            requested_deadline_missed_status: RequestedDeadlineMissedStatus::const_default(),
            requested_incompatible_qos_status: RequestedIncompatibleQosStatus::const_default(),
            sample_rejected_status: SampleRejectedStatus::const_default(),
            subscription_matched_status: SubscriptionMatchedStatus::const_default(),
            matched_publication_list: Vec::new(),
            enabled: false,
            data_available_status_changed_flag: false,
            incompatible_writer_list: Vec::new(),
            status_condition,
            listener_sender,
            listener_mask,
            instances: Vec::new(),
            instance_ownership: Vec::new(),
            transport_reader,
        }
    }

    fn create_indexed_sample_collection(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<IndexedSample>> {
        if let Some(h) = specific_instance_handle {
            if !self.instances.iter().any(|x| x.handle() == h) {
                return Err(DdsError::BadParameter);
            }
        };

        let mut indexed_samples = Vec::new();

        let mut instances_in_collection = Vec::<InstanceState>::new();
        for (index, cache_change) in self.sample_list.iter().enumerate() {
            if let Some(h) = specific_instance_handle {
                if cache_change.instance_handle != h {
                    continue;
                }
            };

            let Some(instance) = self
                .instances
                .iter()
                .find(|x| x.handle == cache_change.instance_handle)
            else {
                continue;
            };

            if !(sample_states.contains(&cache_change.sample_state)
                && view_states.contains(&instance.view_state)
                && instance_states.contains(&instance.instance_state))
            {
                continue;
            }

            if !instances_in_collection
                .iter()
                .any(|x| x.handle() == cache_change.instance_handle)
            {
                instances_in_collection.push(InstanceState::new(cache_change.instance_handle));
            }

            let instance_from_collection = instances_in_collection
                .iter_mut()
                .find(|x| x.handle() == cache_change.instance_handle)
                .expect("Instance must exist");
            instance_from_collection.update_state(cache_change.kind);
            let sample_state = cache_change.sample_state;
            let view_state = instance.view_state;
            let instance_state = instance.instance_state;

            let absolute_generation_rank = (instance.most_recent_disposed_generation_count
                + instance.most_recent_no_writers_generation_count)
                - (instance_from_collection.most_recent_disposed_generation_count
                    + instance_from_collection.most_recent_no_writers_generation_count);

            let (data, valid_data) = match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => {
                    (Some(cache_change.data_value.clone()), true)
                }
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => (None, false),
            };

            let sample_info = SampleInfo {
                sample_state,
                view_state,
                instance_state,
                disposed_generation_count: cache_change.disposed_generation_count,
                no_writers_generation_count: cache_change.no_writers_generation_count,
                sample_rank: 0,     // To be filled up after collection is created
                generation_rank: 0, // To be filled up after collection is created
                absolute_generation_rank,
                source_timestamp: cache_change.source_timestamp,
                instance_handle: cache_change.instance_handle,
                publication_handle: InstanceHandle::new(cache_change.writer_guid),
                valid_data,
            };

            let sample = (data, sample_info);

            indexed_samples.push(IndexedSample { index, sample });

            if indexed_samples.len() as i32 == max_samples {
                break;
            }
        }

        // After the collection is created, update the relative generation rank values and mark the read instances as viewed
        for handle in instances_in_collection.iter().map(|x| x.handle()) {
            let most_recent_sample_absolute_generation_rank = indexed_samples
                .iter()
                .filter(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.instance_handle == handle,
                )
                .map(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.absolute_generation_rank,
                )
                .next_back()
                .expect("Instance handle must exist on collection");

            let mut total_instance_samples_in_collection = indexed_samples
                .iter()
                .filter(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.instance_handle == handle,
                )
                .count();

            for IndexedSample {
                sample: (_, sample_info),
                ..
            } in indexed_samples.iter_mut().filter(
                |IndexedSample {
                     sample: (_, sample_info),
                     ..
                 }| sample_info.instance_handle == handle,
            ) {
                sample_info.generation_rank = sample_info.absolute_generation_rank
                    - most_recent_sample_absolute_generation_rank;

                total_instance_samples_in_collection -= 1;
                sample_info.sample_rank = total_instance_samples_in_collection as i32;
            }

            self.instances
                .iter_mut()
                .find(|x| x.handle() == handle)
                .expect("Sample must exist")
                .mark_viewed()
        }

        if indexed_samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(indexed_samples)
        }
    }

    fn next_instance(&mut self, previous_handle: Option<InstanceHandle>) -> Option<InstanceHandle> {
        match previous_handle {
            Some(p) => self
                .instances
                .iter()
                .map(|x| x.handle())
                .filter(|&h| h > p)
                .min(),
            None => self.instances.iter().map(|x| x.handle()).min(),
        }
    }

    fn convert_cache_change_to_sample(
        &mut self,
        cache_change: CacheChange,
        reception_timestamp: Time,
    ) -> DdsResult<ReaderSample> {
        let instance_handle = {
            match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => {
                    get_instance_handle_from_serialized_foo(
                        cache_change.data_value.as_ref(),
                        self.type_support.as_ref(),
                    )?
                }
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => match cache_change.instance_handle {
                    Some(i) => InstanceHandle::new(i),
                    None => get_instance_handle_from_serialized_key(
                        cache_change.data_value.as_ref(),
                        self.type_support.as_ref(),
                    )?,
                },
            }
        };

        // Update the state of the instance before creating since this has direct impact on
        // the information that is store on the sample
        match cache_change.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == instance_handle)
                {
                    Some(x) => x.update_state(cache_change.kind),
                    None => {
                        let mut s = InstanceState::new(instance_handle);
                        s.update_state(cache_change.kind);
                        self.instances.push(s);
                    }
                }
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == instance_handle)
                {
                    Some(instance) => {
                        instance.update_state(cache_change.kind);
                        Ok(())
                    }
                    None => Err(DdsError::Error(
                        "Received message changing state of unknown instance".to_string(),
                    )),
                }
            }
        }?;
        let instance = self
            .instances
            .iter()
            .find(|x| x.handle() == instance_handle)
            .expect("Sample with handle must exist");
        Ok(ReaderSample {
            kind: cache_change.kind,
            writer_guid: cache_change.writer_guid.into(),
            instance_handle,
            source_timestamp: cache_change.source_timestamp.map(Into::into),
            data_value: cache_change.data_value.clone(),
            sample_state: SampleStateKind::NotRead,
            disposed_generation_count: instance.most_recent_disposed_generation_count,
            no_writers_generation_count: instance.most_recent_no_writers_generation_count,
            reception_timestamp,
        })
    }

    pub fn add_reader_change(
        &mut self,
        cache_change: CacheChange,
        reception_timestamp: Time,
    ) -> DdsResult<AddChangeResult> {
        let sample = self.convert_cache_change_to_sample(cache_change, reception_timestamp)?;
        let change_instance_handle = sample.instance_handle;
        // data_reader exclusive access if the writer is not the allowed to write the sample do an early return
        if self.qos.ownership.kind == OwnershipQosPolicyKind::Exclusive {
            // Get the InstanceHandle of the data writer owning this instance
            if let Some(instance_owner) = self
                .instance_ownership
                .iter()
                .find(|x| x.instance_handle == sample.instance_handle)
            {
                let instance_writer = InstanceHandle::new(sample.writer_guid);
                let Some(sample_owner) = self
                    .matched_publication_list
                    .iter()
                    .find(|x| x.key().value == instance_owner.owner_handle.as_ref())
                else {
                    return Ok(AddChangeResult::NotAdded);
                };
                let Some(sample_writer) = self
                    .matched_publication_list
                    .iter()
                    .find(|x| &x.key().value == instance_writer.as_ref())
                else {
                    return Ok(AddChangeResult::NotAdded);
                };
                if instance_owner.owner_handle != sample.writer_guid
                    && sample_writer.ownership_strength().value
                        <= sample_owner.ownership_strength().value
                {
                    return Ok(AddChangeResult::NotAdded);
                }
            }

            match self
                .instance_ownership
                .iter_mut()
                .find(|x| x.instance_handle == sample.instance_handle)
            {
                Some(x) => {
                    x.owner_handle = sample.writer_guid;
                }
                None => self.instance_ownership.push(InstanceOwnership {
                    instance_handle: sample.instance_handle,
                    owner_handle: sample.writer_guid,
                    last_received_time: reception_timestamp,
                }),
            }
        }

        if matches!(
            sample.kind,
            ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered
        ) {
            if let Some(i) = self
                .instance_ownership
                .iter()
                .position(|x| x.instance_handle == sample.instance_handle)
            {
                self.instance_ownership.remove(i);
            }
        }

        let is_sample_of_interest_based_on_time = {
            let closest_timestamp_before_received_sample = self
                .sample_list
                .iter()
                .filter(|cc| cc.instance_handle == sample.instance_handle)
                .filter(|cc| cc.source_timestamp <= sample.source_timestamp)
                .map(|cc| cc.source_timestamp)
                .max();

            if let Some(Some(t)) = closest_timestamp_before_received_sample {
                if let Some(sample_source_time) = sample.source_timestamp {
                    let sample_separation = sample_source_time - t;
                    DurationKind::Finite(sample_separation)
                        >= self.qos.time_based_filter.minimum_separation
                } else {
                    true
                }
            } else {
                true
            }
        };

        if !is_sample_of_interest_based_on_time {
            return Ok(AddChangeResult::NotAdded);
        }

        let is_max_samples_limit_reached = {
            let total_samples = self
                .sample_list
                .iter()
                .filter(|cc| cc.kind == ChangeKind::Alive)
                .count();

            total_samples == self.qos.resource_limits.max_samples
        };
        let is_max_instances_limit_reached = {
            let mut instance_handle_list = Vec::new();
            for sample_handle in self.sample_list.iter().map(|x| x.instance_handle) {
                if !instance_handle_list.contains(&sample_handle) {
                    instance_handle_list.push(sample_handle);
                }
            }

            if instance_handle_list.contains(&sample.instance_handle) {
                false
            } else {
                instance_handle_list.len() == self.qos.resource_limits.max_instances
            }
        };
        let is_max_samples_per_instance_limit_reached = {
            let total_samples_of_instance = self
                .sample_list
                .iter()
                .filter(|cc| cc.instance_handle == sample.instance_handle)
                .count();

            total_samples_of_instance == self.qos.resource_limits.max_samples_per_instance
        };
        if is_max_samples_limit_reached {
            return Ok(AddChangeResult::Rejected(
                sample.instance_handle,
                SampleRejectedStatusKind::RejectedBySamplesLimit,
            ));
        } else if is_max_instances_limit_reached {
            return Ok(AddChangeResult::Rejected(
                sample.instance_handle,
                SampleRejectedStatusKind::RejectedByInstancesLimit,
            ));
        } else if is_max_samples_per_instance_limit_reached {
            return Ok(AddChangeResult::Rejected(
                sample.instance_handle,
                SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit,
            ));
        }
        let num_alive_samples_of_instance = self
            .sample_list
            .iter()
            .filter(|cc| {
                cc.instance_handle == sample.instance_handle && cc.kind == ChangeKind::Alive
            })
            .count() as u32;

        if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
            if depth == num_alive_samples_of_instance {
                let index_sample_to_remove = self
                    .sample_list
                    .iter()
                    .position(|cc| {
                        cc.instance_handle == sample.instance_handle && cc.kind == ChangeKind::Alive
                    })
                    .expect("Samples must exist");
                self.sample_list.remove(index_sample_to_remove);
            }
        }

        match sample.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == sample.instance_handle)
                {
                    Some(x) => x.update_state(sample.kind),
                    None => {
                        let mut s = InstanceState::new(sample.instance_handle);
                        s.update_state(sample.kind);
                        self.instances.push(s);
                    }
                }
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == sample.instance_handle)
                {
                    Some(instance) => {
                        instance.update_state(sample.kind);
                        Ok(())
                    }
                    None => Err(DdsError::Error(
                        "Received message changing state of unknown instance".to_string(),
                    )),
                }
            }
        }?;

        let sample_writer_guid = sample.writer_guid;
        tracing::debug!(cache_change = ?sample, "Adding change to data reader history cache");
        self.sample_list.push(sample);
        self.data_available_status_changed_flag = true;

        match self.qos.destination_order.kind {
            DestinationOrderQosPolicyKind::BySourceTimestamp => {
                self.sample_list.sort_by(|a, b| {
                    a.source_timestamp
                        .as_ref()
                        .expect("Missing source timestamp")
                        .cmp(
                            b.source_timestamp
                                .as_ref()
                                .expect("Missing source timestamp"),
                        )
                });
            }
            DestinationOrderQosPolicyKind::ByReceptionTimestamp => self
                .sample_list
                .sort_by(|a, b| a.reception_timestamp.cmp(&b.reception_timestamp)),
        }

        match self
            .instance_ownership
            .iter_mut()
            .find(|x| x.instance_handle == change_instance_handle)
        {
            Some(x) => {
                if x.last_received_time < reception_timestamp {
                    x.last_received_time = reception_timestamp;
                }
            }
            None => self.instance_ownership.push(InstanceOwnership {
                instance_handle: change_instance_handle,
                last_received_time: reception_timestamp,
                owner_handle: sample_writer_guid,
            }),
        }
        Ok(AddChangeResult::Added(change_instance_handle))
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn add_matched_publication(
        &mut self,
        publication_builtin_topic_data: PublicationBuiltinTopicData,
    ) {
        match self
            .matched_publication_list
            .iter_mut()
            .find(|x| x.key() == publication_builtin_topic_data.key())
        {
            Some(x) => *x = publication_builtin_topic_data,
            None => self
                .matched_publication_list
                .push(publication_builtin_topic_data),
        }
        self.subscription_matched_status.current_count +=
            self.matched_publication_list.len() as i32;
        self.subscription_matched_status.current_count_change += 1;
        self.subscription_matched_status.total_count += 1;
        self.subscription_matched_status.total_count_change += 1;
    }

    pub fn increment_requested_deadline_missed_status(&mut self, instance_handle: InstanceHandle) {
        self.requested_deadline_missed_status.total_count += 1;
        self.requested_deadline_missed_status.total_count_change += 1;
        self.requested_deadline_missed_status.last_instance_handle = instance_handle;
    }

    pub fn get_requested_deadline_missed_status(&mut self) -> RequestedDeadlineMissedStatus {
        let status = self.requested_deadline_missed_status.clone();
        self.requested_deadline_missed_status.total_count_change = 0;
        status
    }

    pub fn remove_instance_ownership(&mut self, instance_handle: &InstanceHandle) {
        if let Some(i) = self
            .instance_ownership
            .iter()
            .position(|x| &x.instance_handle == instance_handle)
        {
            self.instance_ownership.remove(i);
        }
    }

    pub fn add_requested_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        if !self.incompatible_writer_list.contains(&handle) {
            self.incompatible_writer_list.push(handle);
            self.requested_incompatible_qos_status.total_count += 1;
            self.requested_incompatible_qos_status.total_count_change += 1;
            self.requested_incompatible_qos_status.last_policy_id = incompatible_qos_policy_list[0];
            for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
                if let Some(policy_count) = self
                    .requested_incompatible_qos_status
                    .policies
                    .iter_mut()
                    .find(|x| x.policy_id == incompatible_qos_policy)
                {
                    policy_count.count += 1;
                } else {
                    self.requested_incompatible_qos_status
                        .policies
                        .push(QosPolicyCount {
                            policy_id: incompatible_qos_policy,
                            count: 1,
                        })
                }
            }
        }
    }

    pub fn get_requested_incompatible_qos_status(&mut self) -> RequestedIncompatibleQosStatus {
        let status = self.requested_incompatible_qos_status.clone();
        self.requested_incompatible_qos_status.total_count_change = 0;
        status
    }

    pub fn increment_sample_rejected_status(
        &mut self,
        sample_handle: InstanceHandle,
        sample_rejected_status_kind: SampleRejectedStatusKind,
    ) {
        self.sample_rejected_status.last_instance_handle = sample_handle;
        self.sample_rejected_status.last_reason = sample_rejected_status_kind;
        self.sample_rejected_status.total_count += 1;
        self.sample_rejected_status.total_count_change += 1;
    }

    pub fn get_sample_rejected_status(&mut self) -> SampleRejectedStatus {
        let status = self.sample_rejected_status.clone();
        self.sample_rejected_status.total_count_change = 0;

        status
    }

    pub fn get_subscription_matched_status(&mut self) -> SubscriptionMatchedStatus {
        let status = self.subscription_matched_status.clone();

        self.subscription_matched_status.total_count_change = 0;
        self.subscription_matched_status.current_count_change = 0;

        status
    }

    pub fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect()
    }

    pub fn get_instance_received_time(&self, instance_handle: &InstanceHandle) -> Option<Time> {
        self.instance_ownership
            .iter()
            .find(|x| &x.instance_handle == instance_handle)
            .map(|x| x.last_received_time)
    }

    pub async fn remove_matched_publication(&mut self, publication_handle: &InstanceHandle) {
        let Some(i) = self
            .matched_publication_list
            .iter()
            .position(|x| &x.key().value == publication_handle.as_ref())
        else {
            return;
        };
        self.matched_publication_list.remove(i);
        self.subscription_matched_status.current_count = self.matched_publication_list.len() as i32;
        self.subscription_matched_status.current_count_change -= 1;
        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::AddCommunicationState {
                state: StatusKind::SubscriptionMatched,
            })
            .await;
    }

    pub async fn read(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::DataAvailable,
            })
            .await;

        let indexed_sample_list = self.create_indexed_sample_collection(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )?;

        let change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list
            .into_iter()
            .map(|IndexedSample { index, sample }| (index, sample))
            .unzip();

        for index in change_index_list {
            self.sample_list[index].sample_state = SampleStateKind::Read;
        }

        Ok(samples)
    }

    pub async fn take(
        &mut self,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let indexed_sample_list = self.create_indexed_sample_collection(
            max_samples,
            &sample_states,
            &view_states,
            &instance_states,
            specific_instance_handle,
        )?;

        self.status_condition
            .send_actor_mail(DcpsStatusConditionMail::RemoveCommunicationState {
                state: StatusKind::DataAvailable,
            })
            .await;

        let mut change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list
            .into_iter()
            .map(|IndexedSample { index, sample }| (index, sample))
            .unzip();

        while let Some(index) = change_index_list.pop() {
            self.sample_list.remove(index);
        }

        Ok(samples)
    }

    pub async fn take_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => {
                self.take(
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    Some(next_handle),
                )
                .await
            }
            None => Err(DdsError::NoData),
        }
    }

    pub async fn read_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => {
                self.read(
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    Some(next_handle),
                )
                .await
            }
            None => Err(DdsError::NoData),
        }
    }
}

#[derive(Default)]
pub struct InstanceHandleCounter {
    counter1: u64,
    counter2: u64,
}

impl InstanceHandleCounter {
    pub const fn new() -> Self {
        Self {
            counter1: 0,
            counter2: 0,
        }
    }

    pub fn generate_new_instance_handle(&mut self) -> InstanceHandle {
        if self.counter1 == u64::MAX {
            self.counter2 += 1;
        } else {
            self.counter1 += 1;
        }

        let counter1_bytes = self.counter1.to_ne_bytes();
        let counter2_bytes = self.counter2.to_ne_bytes();

        InstanceHandle::new([
            counter1_bytes[0],
            counter1_bytes[1],
            counter1_bytes[2],
            counter1_bytes[3],
            counter1_bytes[4],
            counter1_bytes[5],
            counter1_bytes[6],
            counter1_bytes[7],
            counter2_bytes[0],
            counter2_bytes[1],
            counter2_bytes[2],
            counter2_bytes[3],
            counter2_bytes[4],
            counter2_bytes[5],
            counter2_bytes[6],
            counter2_bytes[7],
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticipantHandle {
    value: u8,
}

impl ParticipantHandle {
    pub fn new(value: u8) -> Self {
        Self { value }
    }
}

impl From<ParticipantHandle> for InstanceHandle {
    fn from(x: ParticipantHandle) -> Self {
        InstanceHandle::new([x.value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscriberHandle {
    participant_handle: ParticipantHandle,
    value: u8,
}

impl SubscriberHandle {
    pub fn new(participant_handle: ParticipantHandle, value: u8) -> Self {
        Self {
            participant_handle,
            value,
        }
    }
}

impl From<SubscriberHandle> for InstanceHandle {
    fn from(x: SubscriberHandle) -> Self {
        InstanceHandle::new([
            x.participant_handle.value,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublisherHandle {
    participant_handle: ParticipantHandle,
    value: u8,
}

impl PublisherHandle {
    pub fn new(participant_handle: ParticipantHandle, value: u8) -> Self {
        Self {
            participant_handle,
            value,
        }
    }
}

impl From<PublisherHandle> for InstanceHandle {
    fn from(x: PublisherHandle) -> Self {
        InstanceHandle::new([
            x.participant_handle.value,
            0,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopicHandle {
    participant_handle: ParticipantHandle,
    value: u8,
}

impl TopicHandle {
    pub fn new(participant_handle: ParticipantHandle, value: u8) -> Self {
        Self {
            participant_handle,
            value,
        }
    }
}

impl From<TopicHandle> for InstanceHandle {
    fn from(x: TopicHandle) -> Self {
        InstanceHandle::new([
            x.participant_handle.value,
            0,
            0,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataReaderHandle {
    subscriber_handle: SubscriberHandle,
    topic_handle: TopicHandle,
    value: u8,
}

impl DataReaderHandle {
    pub fn new(subscriber_handle: SubscriberHandle, topic_handle: TopicHandle, value: u8) -> Self {
        Self {
            subscriber_handle,
            topic_handle,
            value,
        }
    }
}

impl From<DataReaderHandle> for InstanceHandle {
    fn from(x: DataReaderHandle) -> Self {
        InstanceHandle::new([
            x.subscriber_handle.participant_handle.value,
            x.subscriber_handle.value,
            0,
            x.topic_handle.value,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataWriterHandle {
    publisher_handle: PublisherHandle,
    topic_handle: TopicHandle,
    value: u8,
}

impl DataWriterHandle {
    pub fn new(publisher_handle: PublisherHandle, topic_handle: TopicHandle, value: u8) -> Self {
        Self {
            publisher_handle,
            topic_handle,
            value,
        }
    }
}

impl From<DataWriterHandle> for InstanceHandle {
    fn from(x: DataWriterHandle) -> Self {
        InstanceHandle::new([
            x.publisher_handle.participant_handle.value,
            0,
            x.publisher_handle.value,
            x.topic_handle.value,
            0,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}
