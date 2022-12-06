use std::collections::HashMap;

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::DiscoveredWriterData,
        },
        rtps::{
            messages::submessages::{DataSubmessage, HeartbeatSubmessage},
            stateful_reader::RtpsStatefulReader,
            transport::TransportWrite,
            types::GuidPrefix,
            writer_proxy::RtpsWriterProxy,
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared, DdsWeak},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{DataReaderQos, QosKind},
        qos_policy::{
            DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, Time},
    },
    subscription::{
        data_reader::{DataReader, Sample},
        data_reader_listener::DataReaderListener,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    message_receiver::MessageReceiver, status_condition_impl::StatusConditionImpl,
    topic_impl::TopicImpl, user_defined_subscriber::UserDefinedSubscriber,
};

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(&mut self, reader: &DdsShared<UserDefinedDataReader>);
    fn trigger_on_sample_rejected(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: SampleRejectedStatus,
    );
    fn trigger_on_liveliness_changed(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: SampleLostStatus,
    );
}

impl<Foo> AnyDataReaderListener for Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>
where
    Foo: DdsType + for<'de> DdsDeserialize<'de> + 'static,
{
    fn trigger_on_data_available(&mut self, reader: &DdsShared<UserDefinedDataReader>) {
        self.on_data_available(&DataReader::new(reader.downgrade()))
    }

    fn trigger_on_sample_rejected(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: SampleRejectedStatus,
    ) {
        self.on_sample_rejected(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_liveliness_changed(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: LivelinessChangedStatus,
    ) {
        self.on_liveliness_changed(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: SubscriptionMatchedStatus,
    ) {
        self.on_subscription_matched(&DataReader::new(reader.downgrade()), status)
    }

    fn trigger_on_sample_lost(
        &mut self,
        reader: &DdsShared<UserDefinedDataReader>,
        status: SampleLostStatus,
    ) {
        self.on_sample_lost(&DataReader::new(reader.downgrade()), status)
    }
}

pub struct UserDefinedDataReader {
    rtps_reader: DdsRwLock<RtpsStatefulReader>,
    topic: DdsShared<TopicImpl>,
    listener: DdsRwLock<Option<Box<dyn AnyDataReaderListener + Send + Sync>>>,
    parent_subscriber: DdsWeak<UserDefinedSubscriber>,
    liveliness_changed_status: DdsRwLock<LivelinessChangedStatus>,
    requested_deadline_missed_status: DdsRwLock<RequestedDeadlineMissedStatus>,
    requested_incompatible_qos_status: DdsRwLock<RequestedIncompatibleQosStatus>,
    sample_lost_status: DdsRwLock<SampleLostStatus>,
    sample_rejected_status: DdsRwLock<SampleRejectedStatus>,
    subscription_matched_status: DdsRwLock<SubscriptionMatchedStatus>,
    matched_publication_list: DdsRwLock<HashMap<InstanceHandle, PublicationBuiltinTopicData>>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    user_defined_data_send_condvar: DdsCondvar,
}

impl UserDefinedDataReader {
    pub fn new(
        rtps_reader: RtpsStatefulReader,
        topic: DdsShared<TopicImpl>,
        listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        parent_subscriber: DdsWeak<UserDefinedSubscriber>,
        user_defined_data_send_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedDataReader {
            rtps_reader: DdsRwLock::new(rtps_reader),
            topic,
            listener: DdsRwLock::new(listener),
            parent_subscriber,
            liveliness_changed_status: DdsRwLock::new(LivelinessChangedStatus {
                alive_count: 0,
                not_alive_count: 0,
                alive_count_change: 0,
                not_alive_count_change: 0,
                last_publication_handle: HANDLE_NIL,
            }),
            requested_deadline_missed_status: DdsRwLock::new(RequestedDeadlineMissedStatus {
                total_count: 0,
                total_count_change: 0,
                last_instance_handle: HANDLE_NIL,
            }),
            requested_incompatible_qos_status: DdsRwLock::new(RequestedIncompatibleQosStatus {
                total_count: 0,
                total_count_change: 0,
                last_policy_id: 0,
                policies: Vec::new(),
            }),
            sample_lost_status: DdsRwLock::new(SampleLostStatus {
                total_count: 0,
                total_count_change: 0,
            }),
            sample_rejected_status: DdsRwLock::new(SampleRejectedStatus {
                total_count: 0,
                total_count_change: 0,
                last_reason: SampleRejectedStatusKind::NotRejected,
                last_instance_handle: HANDLE_NIL,
            }),
            subscription_matched_status: DdsRwLock::new(SubscriptionMatchedStatus {
                total_count: 0,
                total_count_change: 0,
                last_publication_handle: HANDLE_NIL,
                current_count: 0,
                current_count_change: 0,
            }),
            matched_publication_list: DdsRwLock::new(HashMap::new()),
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            user_defined_data_send_condvar,
        })
    }
}

impl DdsShared<UserDefinedDataReader> {
    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        self.rtps_reader
            .write_lock()
            .on_data_submessage_received(data_submessage, message_receiver);
    }
}

impl DdsShared<UserDefinedDataReader> {
    pub fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let mut rtps_reader = self.rtps_reader.write_lock();
        rtps_reader.on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        self.user_defined_data_send_condvar.notify_all();
    }
}

impl DdsShared<UserDefinedDataReader> {
    pub fn add_matched_writer(&self, discovered_writer_data: &DiscoveredWriterData) {
        let writer_info = &discovered_writer_data.publication_builtin_topic_data;
        let reader_topic_name = self.topic.get_name();
        let reader_type_name = self.topic.get_type_name();

        if writer_info.topic_name == reader_topic_name && writer_info.type_name == reader_type_name
        {
            let mut rtps_reader_lock = self.rtps_reader.write_lock();
            let reader_qos = rtps_reader_lock.reader().get_qos();
            let parent_subscriber_qos = self.get_subscriber().get_qos();

            let mut incompatible_qos_policy_list = Vec::new();

            if reader_qos.durability < writer_info.durability {
                incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
            }
            if parent_subscriber_qos.presentation.access_scope
                > writer_info.presentation.access_scope
                || parent_subscriber_qos.presentation.coherent_access
                    != writer_info.presentation.coherent_access
                || parent_subscriber_qos.presentation.ordered_access
                    != writer_info.presentation.ordered_access
            {
                incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
            }
            if reader_qos.deadline > writer_info.deadline {
                incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
            }
            if reader_qos.latency_budget > writer_info.latency_budget {
                incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
            }
            if reader_qos.liveliness > writer_info.liveliness {
                incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
            }
            if reader_qos.reliability.kind > writer_info.reliability.kind {
                incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
            }
            if reader_qos.destination_order > writer_info.destination_order {
                incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
            }

            if incompatible_qos_policy_list.is_empty() {
                let writer_proxy = RtpsWriterProxy::new(
                    discovered_writer_data.writer_proxy.remote_writer_guid,
                    discovered_writer_data
                        .writer_proxy
                        .unicast_locator_list
                        .as_ref(),
                    discovered_writer_data
                        .writer_proxy
                        .multicast_locator_list
                        .as_ref(),
                    discovered_writer_data.writer_proxy.data_max_size_serialized,
                    discovered_writer_data.writer_proxy.remote_group_entity_id,
                );

                rtps_reader_lock.matched_writer_add(writer_proxy);

                self.matched_publication_list.write_lock().insert(
                    discovered_writer_data.get_serialized_key().into(),
                    writer_info.clone(),
                );

                // Drop the subscription_matched_status_lock such that the listener can be triggered
                // if needed
                {
                    let mut subscription_matched_status_lock =
                        self.subscription_matched_status.write_lock();
                    subscription_matched_status_lock.total_count += 1;
                    subscription_matched_status_lock.total_count_change += 1;
                    subscription_matched_status_lock.current_count += 1;
                    subscription_matched_status_lock.current_count_change += 1;
                }

                self.status_condition
                    .write_lock()
                    .add_communication_state(StatusKind::SubscriptionMatched);

                if let Some(l) = self.listener.write_lock().as_mut() {
                    self.status_condition
                        .write_lock()
                        .remove_communication_state(StatusKind::SubscriptionMatched);
                    let subscription_matched_status =
                        self.get_subscription_matched_status().unwrap();
                    l.trigger_on_subscription_matched(self, subscription_matched_status)
                };
            } else {
                {
                    let mut requested_incompatible_qos_status_lock =
                        self.requested_incompatible_qos_status.write_lock();
                    requested_incompatible_qos_status_lock.total_count += 1;
                    requested_incompatible_qos_status_lock.total_count_change += 1;
                    requested_incompatible_qos_status_lock.last_policy_id =
                        incompatible_qos_policy_list[0];
                    for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
                        if let Some(policy_count) = requested_incompatible_qos_status_lock
                            .policies
                            .iter_mut()
                            .find(|x| x.policy_id == incompatible_qos_policy)
                        {
                            policy_count.count += 1;
                        } else {
                            requested_incompatible_qos_status_lock
                                .policies
                                .push(QosPolicyCount {
                                    policy_id: incompatible_qos_policy,
                                    count: 1,
                                })
                        }
                    }
                }

                let mut listener_lock = self.listener.write_lock();
                if let Some(l) = listener_lock.as_mut() {
                    let requested_incompatible_qos_status =
                        self.get_requested_incompatible_qos_status().unwrap();
                    l.trigger_on_requested_incompatible_qos(self, requested_incompatible_qos_status)
                }
            }
        }
    }

    pub fn remove_matched_writer(&self, discovered_writer_handle: InstanceHandle) {
        self.rtps_reader
            .write_lock()
            .matched_writer_remove(discovered_writer_handle.into());

        self.matched_publication_list
            .write_lock()
            .remove(&discovered_writer_handle);

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SubscriptionMatched);

        if let Some(l) = self.listener.write_lock().as_mut() {
            self.status_condition
                .write_lock()
                .remove_communication_state(StatusKind::SubscriptionMatched);
            let subscription_matched_status = self.get_subscription_matched_status().unwrap();
            l.trigger_on_subscription_matched(self, subscription_matched_status)
        };
    }
}

impl DdsShared<UserDefinedDataReader> {
    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.write_lock().reader_mut().read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn take<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.write_lock().reader_mut().take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn read_next_sample<Foo>(&self) -> DdsResult<Sample<Foo>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn take_next_sample<Foo>(&self) -> DdsResult<Sample<Foo>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn read_instance<Foo>(
        &self,
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn take_instance<Foo>(
        &self,
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn read_next_instance<Foo>(
        &self,
        _max_samples: i32,
        _previous_handle: Option<InstanceHandle>,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn take_next_instance<Foo>(
        &self,
        _max_samples: i32,
        _previous_handle: Option<InstanceHandle>,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_key_value<Foo>(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn lookup_instance<Foo>(&self, _instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    pub fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut liveliness_changed_status_lock = self.liveliness_changed_status.write_lock();
        let liveliness_changed_status = liveliness_changed_status_lock.clone();

        liveliness_changed_status_lock.alive_count_change = 0;
        liveliness_changed_status_lock.not_alive_count_change = 0;

        Ok(liveliness_changed_status)
    }

    pub fn get_requested_deadline_missed_status(&self) -> DdsResult<RequestedDeadlineMissedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let status = self.requested_deadline_missed_status.read_lock().clone();

        self.requested_deadline_missed_status
            .write_lock()
            .total_count_change = 0;

        Ok(status)
    }

    pub fn get_requested_incompatible_qos_status(
        &self,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut requested_incompatible_qos_status_lock =
            self.requested_incompatible_qos_status.write_lock();
        let requested_incompatible_qos_status = requested_incompatible_qos_status_lock.clone();

        requested_incompatible_qos_status_lock.total_count_change = 0;

        Ok(requested_incompatible_qos_status)
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut sample_lost_status_lock = self.sample_lost_status.write_lock();
        let sample_lost_status = sample_lost_status_lock.clone();

        sample_lost_status_lock.total_count_change = 0;

        Ok(sample_lost_status)
    }

    pub fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut sample_rejected_status_lock = self.sample_rejected_status.write_lock();
        let sample_rejected_status = sample_rejected_status_lock.clone();

        sample_rejected_status_lock.total_count_change = 0;

        Ok(sample_rejected_status)
    }

    pub fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let mut subscription_matched_status_lock = self.subscription_matched_status.write_lock();
        let subscription_matched_status = subscription_matched_status_lock.clone();

        subscription_matched_status_lock.current_count_change = 0;
        subscription_matched_status_lock.total_count_change = 0;

        Ok(subscription_matched_status)
    }

    pub fn get_topicdescription(&self) -> DdsShared<TopicImpl> {
        self.topic.clone()
    }

    pub fn get_subscriber(&self) -> DdsShared<UserDefinedSubscriber> {
        self.parent_subscriber
            .upgrade()
            .expect("Parent subscriber of data reader must exist")
    }

    pub fn wait_for_historical_data(&self, _max_wait: Duration) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.matched_publication_list
            .read_lock()
            .get(&publication_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .matched_publication_list
            .read_lock()
            .iter()
            .map(|(&key, _)| key)
            .collect())
    }
}

impl DdsShared<UserDefinedDataReader> {
    pub fn set_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        let mut rtps_reader_lock = self.rtps_reader.write_lock();
        if *self.enabled.read_lock() {
            rtps_reader_lock
                .reader()
                .get_qos()
                .check_immutability(&qos)?;
        }

        rtps_reader_lock.reader_mut().set_qos(qos)?;

        Ok(())
    }

    pub fn get_qos(&self) -> DdsResult<DataReaderQos> {
        Ok(self.rtps_reader.read_lock().reader().get_qos().clone())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        *self.listener.write_lock() = a_listener;
        Ok(())
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.write_lock().get_enabled_statuses()
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !self.get_subscriber().is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent subscriber disabled".to_string(),
            ));
        }

        self.get_subscriber()
            .get_participant()
            .announce_created_datareader(self.as_discovered_reader_data());
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.read_lock().reader().guid().into()
    }

    fn as_discovered_reader_data(&self) -> DiscoveredReaderData {
        let rtps_reader_lock = self.rtps_reader.read_lock();
        let guid = rtps_reader_lock.reader().guid();
        let reader_qos = rtps_reader_lock.reader().get_qos();
        let topic_qos = self.topic.get_qos();
        let subscriber_qos = self.get_subscriber().get_qos();

        DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                remote_reader_guid: guid,
                remote_group_entity_id: guid.entity_id(),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },

            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey { value: guid.into() },
                participant_key: BuiltInTopicKey { value: [1; 16] },
                topic_name: self.topic.get_name(),
                type_name: self.topic.get_type_name().to_string(),
                durability: reader_qos.durability.clone(),
                deadline: reader_qos.deadline.clone(),
                latency_budget: reader_qos.latency_budget.clone(),
                liveliness: reader_qos.liveliness.clone(),
                reliability: reader_qos.reliability.clone(),
                ownership: reader_qos.ownership.clone(),
                destination_order: reader_qos.destination_order.clone(),
                user_data: reader_qos.user_data.clone(),
                time_based_filter: reader_qos.time_based_filter.clone(),
                presentation: subscriber_qos.presentation.clone(),
                partition: subscriber_qos.partition.clone(),
                topic_data: topic_qos.topic_data,
                group_data: subscriber_qos.group_data,
            },
        }
    }
}

impl DdsShared<UserDefinedDataReader> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        self.rtps_reader.write_lock().send_message(transport);
    }

    pub fn update_communication_status(&self, now: Time) {
        if self
            .rtps_reader
            .write_lock()
            .reader_mut()
            .take_data_available()
        {
            self.on_data_available()
        };

        if !self
            .rtps_reader
            .write_lock()
            .reader_mut()
            .get_deadline_missed_instances(now)
            .is_empty()
        {
            self.on_requested_deadline_missed()
        }
    }

    fn on_data_available(&self) {
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::DataAvailable);

        if let Some(listener) = self.listener.write_lock().as_mut() {
            listener.trigger_on_data_available(self);
        }
    }

    fn on_requested_deadline_missed(&self) {
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::RequestedDeadlineMissed);

        if let Some(listener) = self.listener.write_lock().as_mut() {
            let status = self.get_requested_deadline_missed_status().unwrap();
            listener.trigger_on_requested_deadline_missed(self, status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            reader::RtpsReader,
            types::{EntityId, EntityKind, Guid, TopicKind, ENTITYID_UNKNOWN, GUID_UNKNOWN},
        },
        infrastructure::time::DURATION_ZERO,
        infrastructure::{
            qos::{SubscriberQos, TopicQos},
            qos_policy::{
                DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
                GroupDataQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy,
                OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
                ReliabilityQosPolicy, ReliabilityQosPolicyKind, TopicDataQosPolicy,
                UserDataQosPolicy,
            },
        },
        topic_definition::type_support::DdsSerialize,
    };
    use crate::{
        implementation::{
            data_representation_builtin_endpoints::discovered_writer_data::WriterProxy,
            dds_impl::topic_impl::TopicImpl, rtps::group::RtpsGroupImpl,
            utils::shared_object::DdsShared,
        },
        topic_definition::type_support::{DdsType, Endianness},
    };

    use mockall::mock;
    use std::io::Write;

    struct UserData(u8);

    impl DdsType for UserData {
        fn type_name() -> &'static str {
            "UserData"
        }

        fn has_key() -> bool {
            false
        }
    }

    impl<'de> DdsDeserialize<'de> for UserData {
        fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
            Ok(UserData(buf[0]))
        }
    }

    impl DdsSerialize for UserData {
        fn serialize<W: Write, E: Endianness>(&self, mut writer: W) -> DdsResult<()> {
            writer
                .write(&[self.0])
                .map(|_| ())
                .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
        }
    }

    mock! {
        Listener {}
        impl AnyDataReaderListener for Listener {
            fn trigger_on_data_available(&mut self, reader: &DdsShared<UserDefinedDataReader>);
            fn trigger_on_sample_rejected(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
                status: SampleRejectedStatus,
            );
            fn trigger_on_liveliness_changed(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
                status: LivelinessChangedStatus,
            );
            fn trigger_on_requested_deadline_missed(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
                status: RequestedDeadlineMissedStatus,
            );
            fn trigger_on_requested_incompatible_qos(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
                status: RequestedIncompatibleQosStatus,
            );
            fn trigger_on_subscription_matched(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
                status: SubscriptionMatchedStatus,
            );
            fn trigger_on_sample_lost(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
                status: SampleLostStatus,
            );
        }
    }

    #[test]
    fn get_instance_handle() {
        let guid = Guid::new(
            GuidPrefix::from([4; 12]),
            EntityId::new([3; 3], EntityKind::BuiltInParticipant),
        );
        let dummy_topic = TopicImpl::new(GUID_UNKNOWN, TopicQos::default(), "", "", DdsWeak::new());
        let qos = DataReaderQos::default();
        let stateful_reader = RtpsStatefulReader::new(RtpsReader::new::<UserData>(
            RtpsEndpoint::new(guid, TopicKind::NoKey, &[], &[]),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            qos,
        ));

        let data_reader: DdsShared<UserDefinedDataReader> = UserDefinedDataReader::new(
            stateful_reader,
            dummy_topic,
            None,
            DdsWeak::new(),
            DdsCondvar::new(),
        );
        *data_reader.enabled.write_lock() = true;

        let expected_instance_handle: InstanceHandle = guid.into();
        let instance_handle = data_reader.get_instance_handle();
        assert_eq!(expected_instance_handle, instance_handle);
    }

    #[test]
    fn add_compatible_matched_writer() {
        let type_name = "test_type";
        let topic_name = "test_topic".to_string();
        let parent_subscriber = UserDefinedSubscriber::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
            DdsCondvar::new(),
        );
        let test_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            type_name,
            &topic_name,
            DdsWeak::new(),
        );

        let rtps_reader = RtpsStatefulReader::new(RtpsReader::new::<UserData>(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            DataReaderQos::default(),
        ));

        let data_reader = UserDefinedDataReader::new(
            rtps_reader,
            test_topic,
            None,
            parent_subscriber.downgrade(),
            DdsCondvar::new(),
        );
        *data_reader.enabled.write_lock() = true;
        let publication_builtin_topic_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey { value: [2; 16] },
            participant_key: BuiltInTopicKey { value: [1; 16] },
            topic_name: topic_name.clone(),
            type_name: type_name.to_string(),
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(),
            liveliness: LivelinessQosPolicy::default(),
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: crate::infrastructure::time::Duration::new(0, 0),
            },
            ownership: OwnershipQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            user_data: UserDataQosPolicy::default(),
            presentation: PresentationQosPolicy::default(),
            partition: PartitionQosPolicy::default(),
            topic_data: TopicDataQosPolicy::default(),
            group_data: GroupDataQosPolicy::default(),
            lifespan: LifespanQosPolicy::default(),
        };
        let remote_writer_guid = Guid::new(
            GuidPrefix::from([2; 12]),
            EntityId::new([2; 3], EntityKind::UserDefinedWriterWithKey),
        );
        let discovered_writer_data = DiscoveredWriterData {
            writer_proxy: WriterProxy {
                remote_writer_guid,
                remote_group_entity_id: ENTITYID_UNKNOWN,
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
            },
            publication_builtin_topic_data: publication_builtin_topic_data.clone(),
        };
        data_reader.add_matched_writer(&discovered_writer_data);

        let subscription_matched_status = data_reader.get_subscription_matched_status().unwrap();
        assert_eq!(subscription_matched_status.current_count, 1);
        assert_eq!(subscription_matched_status.current_count_change, 1);
        assert_eq!(subscription_matched_status.total_count, 1);
        assert_eq!(subscription_matched_status.total_count_change, 1);

        let matched_publications = data_reader.get_matched_publications().unwrap();
        assert_eq!(matched_publications.len(), 1);
        assert_eq!(matched_publications[0], remote_writer_guid.into());
        let matched_publication_data = data_reader
            .get_matched_publication_data(matched_publications[0])
            .unwrap();
        assert_eq!(matched_publication_data, publication_builtin_topic_data);
    }

    #[test]
    fn add_incompatible_matched_writer() {
        let type_name = "test_type";
        let topic_name = "test_topic".to_string();
        let parent_subscriber = UserDefinedSubscriber::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsWeak::new(),
            DdsCondvar::new(),
        );
        let test_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            type_name,
            &topic_name,
            DdsWeak::new(),
        );

        let mut data_reader_qos = DataReaderQos::default();
        data_reader_qos.reliability.kind = ReliabilityQosPolicyKind::Reliable;

        let rtps_reader = RtpsStatefulReader::new(RtpsReader::new::<UserData>(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            data_reader_qos,
        ));

        let data_reader = UserDefinedDataReader::new(
            rtps_reader,
            test_topic,
            None,
            parent_subscriber.downgrade(),
            DdsCondvar::new(),
        );
        *data_reader.enabled.write_lock() = true;
        let publication_builtin_topic_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey { value: [2; 16] },
            participant_key: BuiltInTopicKey { value: [1; 16] },
            topic_name: topic_name.clone(),
            type_name: type_name.to_string(),
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(),
            liveliness: LivelinessQosPolicy::default(),
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: crate::infrastructure::time::Duration::new(0, 0),
            },
            ownership: OwnershipQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            user_data: UserDataQosPolicy::default(),
            presentation: PresentationQosPolicy::default(),
            partition: PartitionQosPolicy::default(),
            topic_data: TopicDataQosPolicy::default(),
            group_data: GroupDataQosPolicy::default(),
            lifespan: LifespanQosPolicy::default(),
        };
        let discovered_writer_data = DiscoveredWriterData {
            writer_proxy: WriterProxy {
                remote_writer_guid: Guid::new(
                    GuidPrefix::from([2; 12]),
                    EntityId::new([2; 3], EntityKind::UserDefinedWriterWithKey),
                ),
                remote_group_entity_id: ENTITYID_UNKNOWN,
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                data_max_size_serialized: None,
            },
            publication_builtin_topic_data: publication_builtin_topic_data.clone(),
        };
        data_reader.add_matched_writer(&discovered_writer_data);

        let matched_publications = data_reader.get_matched_publications().unwrap();
        assert_eq!(matched_publications.len(), 0);

        let requested_incompatible_qos_status =
            data_reader.get_requested_incompatible_qos_status().unwrap();
        assert_eq!(requested_incompatible_qos_status.total_count, 1);
        assert_eq!(requested_incompatible_qos_status.total_count_change, 1);
        assert_eq!(
            requested_incompatible_qos_status.last_policy_id,
            RELIABILITY_QOS_POLICY_ID
        );
        assert_eq!(
            requested_incompatible_qos_status.policies,
            vec![QosPolicyCount {
                policy_id: RELIABILITY_QOS_POLICY_ID,
                count: 1,
            }]
        )
    }
}
