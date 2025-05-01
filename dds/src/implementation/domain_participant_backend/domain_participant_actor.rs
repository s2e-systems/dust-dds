use fnmatch_regex::glob_to_regex;

use super::{
    domain_participant_actor_mail::{DomainParticipantMail, EventServiceMail, MessageServiceMail},
    entities::{
        data_writer::{DataWriterEntity, TransportWriterKind},
        domain_participant::DomainParticipantEntity,
        publisher::PublisherEntity,
    },
    handle::InstanceHandleCounter,
};
use crate::{
    builtin_topics::{
        BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        DCPS_PUBLICATION,
    },
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
        domain_participant::DomainParticipantAsync, publisher::PublisherAsync,
        publisher_listener::PublisherListenerAsync, subscriber::SubscriberAsync, topic::TopicAsync,
    },
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        domain_participant_factory::domain_participant_factory_actor::DdsTransportParticipant,
        listeners::{
            data_writer_listener::{self, DataWriterListenerActor},
            domain_participant_listener,
            publisher_listener::{self, PublisherListenerActor},
        },
        status_condition::status_condition_actor::{self, StatusConditionActor},
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_serialized_key_from_serialized_foo,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        qos_policy::{
            DurabilityQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind,
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::StatusKind,
        time::{Duration, DurationKind, Time},
    },
    runtime::{
        actor::{Actor, ActorAddress},
        executor::Executor,
        timer::TimerDriver,
    },
    topic_definition::type_support::DdsSerialize,
    transport::{
        self,
        types::{
            DurabilityKind, EntityId, ReliabilityKind, TopicKind, ENTITYID_UNKNOWN,
            USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY,
        },
    },
    xtypes::dynamic_type::DynamicType,
};

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
                            .send_actor_mail(DomainParticipantMail::MessageService(
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
                        .send_actor_mail(DomainParticipantMail::EventService(
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
