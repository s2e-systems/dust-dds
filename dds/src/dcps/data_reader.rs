use super::{
    builtin_topics::PublicationBuiltinTopicData,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataReaderQos,
        qos_policy::{
            DestinationOrderQosPolicyKind, HistoryQosPolicyKind, OwnershipQosPolicyKind,
            QosPolicyId,
        },
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::{DurationKind, Time},
    },
    status_condition::StatusCondition,
    xtypes_glue::key_and_instance_handle::{
        get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
    },
};
use crate::{
    transport::{
        history_cache::CacheChange,
        reader::{TransportStatefulReader, TransportStatelessReader},
        types::{ChangeKind, Guid},
    },
    xtypes::dynamic_type::DynamicType,
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

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

struct InstanceReceivedTime {
    instance_handle: InstanceHandle,
    last_received_time: Time,
}

struct InstanceOwnership {
    instance_handle: InstanceHandle,
    owner_handle: [u8; 16],
}

pub struct DataReaderEntity<S, L> {
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
    matched_publication_list: Vec<PublicationBuiltinTopicData>,
    enabled: bool,
    data_available_status_changed_flag: bool,
    incompatible_writer_list: Vec<InstanceHandle>,
    status_condition: S,
    listener_sender: L,
    listener_mask: Vec<StatusKind>,
    instances: Vec<InstanceState>,
    instance_received_time: Vec<InstanceReceivedTime>,
    instance_ownership: Vec<InstanceOwnership>,
    transport_reader: TransportReaderKind,
}

impl<S, L> DataReaderEntity<S, L> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        type_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        status_condition: S,
        listener_sender: L,
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
            matched_publication_list: Vec::new(),
            enabled: false,
            data_available_status_changed_flag: false,
            incompatible_writer_list: Vec::new(),
            status_condition,
            listener_sender,
            listener_mask,
            instances: Vec::new(),
            instance_received_time: Vec::new(),
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
                    .find(|x| &x.key().value == instance_owner.owner_handle.as_ref())
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
                Some(x) => x.owner_handle = sample.writer_guid,
                None => self.instance_ownership.push(InstanceOwnership {
                    instance_handle: sample.instance_handle,
                    owner_handle: sample.writer_guid,
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
            .instance_received_time
            .iter_mut()
            .find(|x| x.instance_handle == change_instance_handle)
        {
            Some(x) => {
                if x.last_received_time < reception_timestamp {
                    x.last_received_time = reception_timestamp;
                }
            }
            None => self.instance_received_time.push(InstanceReceivedTime {
                instance_handle: change_instance_handle,
                last_received_time: reception_timestamp,
            }),
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

    pub fn status_condition(&self) -> &S {
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
        self.matched_publication_list
            .iter()
            .find(|x| &x.key().value == handle.as_ref())
    }

    pub fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect()
    }

    pub fn listener(&self) -> &L {
        &self.listener_sender
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }

    pub fn set_listener(&mut self, listener_sender: L, listener_mask: Vec<StatusKind>) {
        self.listener_sender = listener_sender;
        self.listener_mask = listener_mask;
    }

    pub fn get_instance_received_time(&self, instance_handle: &InstanceHandle) -> Option<Time> {
        self.instance_received_time
            .iter()
            .find(|x| &x.instance_handle == instance_handle)
            .map(|x| x.last_received_time)
    }
}

impl<S, L> DataReaderEntity<S, L>
where
    S: StatusCondition,
{
    pub fn remove_matched_publication(&mut self, publication_handle: &InstanceHandle) {
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
            .add_state(StatusKind::SubscriptionMatched);
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
            .remove_state(StatusKind::DataAvailable);

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
            .remove_state(StatusKind::DataAvailable);

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
}
