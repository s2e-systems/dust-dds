use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::{
        listeners::data_reader_listener::DataReaderListenerActor,
        status_condition::status_condition_actor::{self, StatusConditionActor},
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataReaderQos,
        qos_policy::{
            DestinationOrderQosPolicyKind, HistoryQosPolicyKind, OwnershipQosPolicyKind,
            QosPolicyId,
        },
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::{DurationKind, Time},
    },
    runtime::{actor::Actor, executor::TaskHandle},
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    transport::{
        history_cache::CacheChange,
        reader::{TransportStatefulReader, TransportStatelessReader},
        types::{ChangeKind, Guid},
    },
    xtypes::dynamic_type::DynamicType,
};

type SampleList = Vec<(Option<Arc<[u8]>>, SampleInfo)>;

pub enum AddChangeResult {
    Added(InstanceHandle),
    NotAdded,
    Rejected(InstanceHandle, SampleRejectedStatusKind),
}

struct InstanceState {
    view_state: ViewStateKind,
    instance_state: InstanceStateKind,
    most_recent_disposed_generation_count: i32,
    most_recent_no_writers_generation_count: i32,
}

impl InstanceState {
    fn new() -> Self {
        Self {
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

pub enum TransportReaderKind {
    Stateful(Box<dyn TransportStatefulReader>),
    Stateless(Box<dyn TransportStatelessReader>),
}

impl TransportReaderKind {
    pub fn guid(&self) -> Guid {
        match self {
            TransportReaderKind::Stateful(r) => r.guid(),
            TransportReaderKind::Stateless(r) => r.guid(),
        }
    }
}

pub struct DataReaderEntity {
    instance_handle: InstanceHandle,
    sample_list: Vec<ReaderSample>,
    qos: DataReaderQos,
    topic_name: String,
    type_name: String,
    type_support: Arc<dyn DynamicType + Send + Sync>,
    _liveliness_changed_status: LivelinessChangedStatus,
    requested_deadline_missed_status: RequestedDeadlineMissedStatus,
    requested_incompatible_qos_status: RequestedIncompatibleQosStatus,
    _sample_lost_status: SampleLostStatus,
    sample_rejected_status: SampleRejectedStatus,
    subscription_matched_status: SubscriptionMatchedStatus,
    matched_publication_list: HashMap<InstanceHandle, PublicationBuiltinTopicData>,
    enabled: bool,
    data_available_status_changed_flag: bool,
    incompatible_writer_list: HashSet<InstanceHandle>,
    status_condition: Actor<StatusConditionActor>,
    listener: Option<Actor<DataReaderListenerActor>>,
    listener_mask: Vec<StatusKind>,
    instances: HashMap<InstanceHandle, InstanceState>,
    instance_deadline_missed_task: HashMap<InstanceHandle, TaskHandle>,
    instance_ownership: HashMap<InstanceHandle, [u8; 16]>,
    transport_reader: TransportReaderKind,
}

impl DataReaderEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        type_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        status_condition: Actor<StatusConditionActor>,
        listener: Option<Actor<DataReaderListenerActor>>,
        listener_mask: Vec<StatusKind>,
        transport_reader: TransportReaderKind,
    ) -> Self {
        Self {
            instance_handle,
            sample_list: Vec::new(),
            qos,
            topic_name,
            type_name,
            type_support,
            _liveliness_changed_status: LivelinessChangedStatus::default(),
            requested_deadline_missed_status: RequestedDeadlineMissedStatus::default(),
            requested_incompatible_qos_status: RequestedIncompatibleQosStatus::default(),
            _sample_lost_status: SampleLostStatus::default(),
            sample_rejected_status: SampleRejectedStatus::default(),
            subscription_matched_status: SubscriptionMatchedStatus::default(),
            matched_publication_list: HashMap::new(),
            enabled: false,
            data_available_status_changed_flag: false,
            incompatible_writer_list: HashSet::new(),
            status_condition,
            listener,
            listener_mask,
            instances: HashMap::new(),
            instance_deadline_missed_task: HashMap::new(),
            instance_ownership: HashMap::new(),
            transport_reader,
        }
    }

    pub fn read(
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
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::DataAvailable,
            });

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

    pub fn take(
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
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::DataAvailable,
            });

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

    fn create_indexed_sample_collection(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<IndexedSample>> {
        if let Some(h) = specific_instance_handle {
            if !self.instances.contains_key(&h) {
                return Err(DdsError::BadParameter);
            }
        };

        let mut indexed_samples = Vec::new();

        let instances = &self.instances;
        let mut instances_in_collection = HashMap::new();
        for (index, cache_change) in self
            .sample_list
            .iter()
            .enumerate()
            .filter(|(_, cc)| {
                sample_states.contains(&cc.sample_state)
                    && view_states.contains(&instances[&cc.instance_handle].view_state)
                    && instance_states.contains(&instances[&cc.instance_handle].instance_state)
                    && if let Some(h) = specific_instance_handle {
                        h == cc.instance_handle
                    } else {
                        true
                    }
            })
            .take(max_samples as usize)
        {
            instances_in_collection
                .entry(cache_change.instance_handle)
                .or_insert_with(InstanceState::new);

            instances_in_collection
                .get_mut(&cache_change.instance_handle)
                .unwrap()
                .update_state(cache_change.kind);
            let sample_state = cache_change.sample_state;
            let view_state = self.instances[&cache_change.instance_handle].view_state;
            let instance_state = self.instances[&cache_change.instance_handle].instance_state;

            let absolute_generation_rank = (self.instances[&cache_change.instance_handle]
                .most_recent_disposed_generation_count
                + self.instances[&cache_change.instance_handle]
                    .most_recent_no_writers_generation_count)
                - (instances_in_collection[&cache_change.instance_handle]
                    .most_recent_disposed_generation_count
                    + instances_in_collection[&cache_change.instance_handle]
                        .most_recent_no_writers_generation_count);

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

            indexed_samples.push(IndexedSample { index, sample })
        }

        // After the collection is created, update the relative generation rank values and mark the read instances as viewed
        for handle in instances_in_collection.into_keys() {
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
                .last()
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
                .get_mut(&handle)
                .expect("Sample must exist on hash map")
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
            Some(p) => self.instances.keys().filter(|&h| h > &p).min().cloned(),
            None => self.instances.keys().min().cloned(),
        }
    }

    pub fn take_next_instance(
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
            Some(next_handle) => self.take(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }

    pub fn read_next_instance(
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
            Some(next_handle) => self.read(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                Some(next_handle),
            ),
            None => Err(DdsError::NoData),
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
                self.instances
                    .entry(instance_handle)
                    .or_insert_with(InstanceState::new)
                    .update_state(cache_change.kind);
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self.instances.get_mut(&instance_handle) {
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

        Ok(ReaderSample {
            kind: cache_change.kind,
            writer_guid: cache_change.writer_guid.into(),
            instance_handle,
            source_timestamp: cache_change.source_timestamp.map(Into::into),
            data_value: cache_change.data_value.clone(),
            sample_state: SampleStateKind::NotRead,
            disposed_generation_count: self.instances[&instance_handle]
                .most_recent_disposed_generation_count,
            no_writers_generation_count: self.instances[&instance_handle]
                .most_recent_no_writers_generation_count,
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
            if let Some(&instance_owner_handle) =
                self.instance_ownership.get(&sample.instance_handle)
            {
                let instance_owner = InstanceHandle::new(instance_owner_handle);
                let instance_writer = InstanceHandle::new(sample.writer_guid);
                if instance_owner_handle != sample.writer_guid
                    && self.matched_publication_list[&instance_writer]
                        .ownership_strength()
                        .value
                        <= self.matched_publication_list[&instance_owner]
                            .ownership_strength()
                            .value
                {
                    return Ok(AddChangeResult::NotAdded);
                }
            }

            self.instance_ownership
                .insert(sample.instance_handle, sample.writer_guid);
        }

        if matches!(
            sample.kind,
            ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered
        ) {
            self.instance_ownership.remove(&sample.instance_handle);
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
            let instance_handle_list: HashSet<_> = self
                .sample_list
                .iter()
                .map(|cc| cc.instance_handle)
                .collect();

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
                self.instances
                    .entry(sample.instance_handle)
                    .or_insert_with(InstanceState::new)
                    .update_state(sample.kind);
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self.instances.get_mut(&sample.instance_handle) {
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

        if let Some(t) = self
            .instance_deadline_missed_task
            .remove(&change_instance_handle)
        {
            t.abort();
        }

        Ok(AddChangeResult::Added(change_instance_handle))
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn qos(&self) -> &DataReaderQos {
        &self.qos
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        qos.is_consistent()?;
        if self.enabled {
            self.qos.check_immutability(&qos)?
        }

        self.qos = qos;

        Ok(())
    }

    pub fn add_matched_publication(
        &mut self,
        publication_builtin_topic_data: PublicationBuiltinTopicData,
    ) {
        self.matched_publication_list.insert(
            InstanceHandle::new(publication_builtin_topic_data.key.value),
            publication_builtin_topic_data,
        );
        self.subscription_matched_status.current_count +=
            self.matched_publication_list.len() as i32;
        self.subscription_matched_status.current_count_change += 1;
        self.subscription_matched_status.total_count += 1;
        self.subscription_matched_status.total_count_change += 1;
    }

    pub fn remove_matched_publication(&mut self, publication_handle: &InstanceHandle) {
        self.matched_publication_list.remove(publication_handle);
        self.subscription_matched_status.current_count = self.matched_publication_list.len() as i32;
        self.subscription_matched_status.current_count_change -= 1;
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::SubscriptionMatched,
            });
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
        self.instance_ownership.remove(instance_handle);
    }

    pub fn add_requested_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        if !self.incompatible_writer_list.contains(&handle) {
            self.incompatible_writer_list.insert(handle);
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

    pub fn status_condition(&self) -> &Actor<StatusConditionActor> {
        &self.status_condition
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

    pub fn transport_reader(&self) -> &TransportReaderKind {
        &self.transport_reader
    }

    pub fn transport_reader_mut(&mut self) -> &mut TransportReaderKind {
        &mut self.transport_reader
    }

    pub fn get_matched_publication_data(
        &self,
        handle: &InstanceHandle,
    ) -> Option<&PublicationBuiltinTopicData> {
        self.matched_publication_list.get(handle)
    }

    pub fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list.keys().cloned().collect()
    }

    pub fn insert_instance_deadline_missed_task(
        &mut self,
        instance_handle: InstanceHandle,
        task: TaskHandle,
    ) {
        self.instance_deadline_missed_task
            .insert(instance_handle, task);
    }

    pub fn listener(&self) -> Option<&Actor<DataReaderListenerActor>> {
        self.listener.as_ref()
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }

    pub fn set_listener(
        &mut self,
        listener: Option<Actor<DataReaderListenerActor>>,
        listener_mask: Vec<StatusKind>,
    ) {
        self.listener = listener;
        self.listener_mask = listener_mask;
    }
}
