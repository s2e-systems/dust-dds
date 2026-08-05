use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataReaderQos,
        qos_policy::{DestinationOrderQosPolicyKind, HistoryQosPolicyKind, OwnershipQosPolicyKind},
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
        status::SampleRejectedStatusKind,
        time::{DurationKind, TIME_INVALID_NSEC, TIME_INVALID_SEC, Time},
    },
    transport::types::{ChangeKind, Guid},
};
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

pub type SampleList = Vec<(Arc<[u8]>, SampleInfo)>;

pub enum AddChangeResult {
    Added,
    NotAdded,
    Rejected(InstanceHandle, SampleRejectedStatusKind),
}

pub struct InstanceState {
    pub handle: InstanceHandle,
    view_state: ViewStateKind,
    instance_state: InstanceStateKind,
    most_recent_disposed_generation_count: i32,
    most_recent_no_writers_generation_count: i32,
    last_received_time_stamp: Time,
}

impl InstanceState {
    pub fn new(handle: InstanceHandle) -> Self {
        Self {
            handle,
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
            last_received_time_stamp: Time::new(TIME_INVALID_SEC, TIME_INVALID_NSEC),
        }
    }

    pub fn update_state(&mut self, change_kind: ChangeKind, now: Option<Time>) {
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
        if let Some(t) = now {
            self.last_received_time_stamp = t;
        }
    }

    pub fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }

    pub fn handle(&self) -> &InstanceHandle {
        &self.handle
    }

    pub fn last_received_time_stamp(&self) -> Time {
        self.last_received_time_stamp
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
}

pub struct InstanceOwnership {
    pub instance_handle: InstanceHandle,
    pub owner_handle: [u8; 16],
    pub last_received_time: Time,
}

pub struct DataReaderEntity<T> {
    pub instance_handle: InstanceHandle,
    pub sample_list: Vec<ReaderSample>,
    pub qos: DataReaderQos,
    pub topic_name: String,
    pub matched_publication_list: Vec<PublicationBuiltinTopicData>,
    pub enabled: bool,
    pub instances: Vec<InstanceState>,
    pub instance_ownership: Vec<InstanceOwnership>,
    pub transport_reader: T,
}

impl<T> DataReaderEntity<T> {
    pub fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        transport_reader: T,
    ) -> Self {
        Self {
            instance_handle,
            sample_list: Vec::new(),
            qos,
            topic_name,
            matched_publication_list: Vec::new(),
            enabled: false,
            instances: Vec::new(),
            instance_ownership: Vec::new(),
            transport_reader,
        }
    }

    pub fn create_sample_collection(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: &Option<InstanceHandle>,
        take: bool,
    ) -> DdsResult<SampleList> {
        if let Some(h) = specific_instance_handle {
            if !self.instances.iter().any(|x| x.handle() == h) {
                return Err(DdsError::BadParameter);
            }
        };

        let mut samples = Vec::new();

        let mut instances_in_collection = Vec::<InstanceState>::new();
        self.sample_list.retain_mut(|cache_change| {
            if samples.len() as i32 == max_samples {
                return true;
            }

            if let Some(h) = specific_instance_handle {
                if &cache_change.instance_handle != h {
                    return true;
                }
            };

            let Some(instance) = self
                .instances
                .iter()
                .find(|x| x.handle == cache_change.instance_handle)
            else {
                return true;
            };

            if !(sample_states.contains(&cache_change.sample_state)
                && view_states.contains(&instance.view_state)
                && instance_states.contains(&instance.instance_state))
            {
                return true;
            }

            if !instances_in_collection
                .iter()
                .any(|x| x.handle() == &cache_change.instance_handle)
            {
                instances_in_collection.push(InstanceState::new(cache_change.instance_handle));
            }

            let instance_from_collection = instances_in_collection
                .iter_mut()
                .find(|x| x.handle() == &cache_change.instance_handle)
                .expect("Instance must exist");
            instance_from_collection.update_state(cache_change.kind, None);
            let sample_state = cache_change.sample_state;
            let view_state = instance.view_state;
            let instance_state = instance.instance_state;

            let absolute_generation_rank = (instance.most_recent_disposed_generation_count
                + instance.most_recent_no_writers_generation_count)
                - (instance_from_collection.most_recent_disposed_generation_count
                    + instance_from_collection.most_recent_no_writers_generation_count);

            let (data, valid_data) = match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => {
                    (cache_change.data_value.clone(), true)
                }
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => {
                    (cache_change.data_value.clone(), false)
                }
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

            samples.push((data, sample_info));

            if take {
                false
            } else {
                cache_change.sample_state = SampleStateKind::Read;
                true
            }
        });

        // After the collection is created, update the relative generation rank values and mark the read instances as viewed
        for handle in instances_in_collection.iter().map(|x| x.handle()) {
            let most_recent_sample_absolute_generation_rank = samples
                .iter()
                .filter(|(_, sample_info)| &sample_info.instance_handle == handle)
                .map(|(_, sample_info)| sample_info.absolute_generation_rank)
                .next_back()
                .expect("Instance handle must exist on collection");

            let mut total_instance_samples_in_collection = samples
                .iter()
                .filter(|(_, sample_info)| &sample_info.instance_handle == handle)
                .count();

            for (_, sample_info) in samples
                .iter_mut()
                .filter(|(_, sample_info)| &sample_info.instance_handle == handle)
            {
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

        if samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(samples)
        }
    }

    pub fn next_instance(
        &mut self,
        previous_handle: &Option<InstanceHandle>,
    ) -> Option<InstanceHandle> {
        match previous_handle {
            Some(p) => self
                .instances
                .iter()
                .map(|x| x.handle())
                .filter(|&h| h > p)
                .min()
                .cloned(),
            None => self.instances.iter().map(|x| x.handle()).min().cloned(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn add_reader_change(
        &mut self,
        writer_guid: Guid,
        data_value: Arc<[u8]>,
        change_kind: ChangeKind,
        change_instance_handle: [u8; 16],
        change_source_timestamp: Option<Time>,
        reception_timestamp: Time,
    ) -> DdsResult<AddChangeResult> {
        let instance_handle = InstanceHandle::new(change_instance_handle);
        // Update the state of the instance before creating since this has direct impact on
        // the information that is stored on the sample
        match change_kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                match self
                    .instances
                    .iter_mut()
                    .find(|x| x.handle() == &instance_handle)
                {
                    Some(x) => x.update_state(change_kind, Some(reception_timestamp)),
                    None => {
                        let mut s = InstanceState::new(instance_handle);
                        s.update_state(change_kind, Some(reception_timestamp));
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
                    .find(|x| x.handle() == &instance_handle)
                {
                    Some(instance) => {
                        instance.update_state(change_kind, Some(reception_timestamp));
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
            .find(|x| x.handle() == &instance_handle)
            .expect("Sample with handle must exist");
        let sample = ReaderSample {
            kind: change_kind,
            writer_guid: writer_guid.into(),
            instance_handle,
            source_timestamp: change_source_timestamp,
            data_value,
            sample_state: SampleStateKind::NotRead,
            disposed_generation_count: instance.most_recent_disposed_generation_count,
            no_writers_generation_count: instance.most_recent_no_writers_generation_count,
        };

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
                    .find(|x| x.handle() == &sample.instance_handle)
                {
                    Some(x) => x.update_state(sample.kind, Some(reception_timestamp)),
                    None => {
                        let mut s = InstanceState::new(sample.instance_handle);
                        s.update_state(sample.kind, Some(reception_timestamp));
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
                    .find(|x| x.handle() == &sample.instance_handle)
                {
                    Some(instance) => {
                        instance.update_state(sample.kind, Some(reception_timestamp));
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

        match self.qos.destination_order.kind {
            DestinationOrderQosPolicyKind::BySourceTimestamp => {
                // Insert the element at the place where the first source timestamp is bigger than the currently received one
                let insert_position = self
                    .sample_list
                    .iter()
                    .position(|x| x.source_timestamp > sample.source_timestamp)
                    .unwrap_or(0);
                self.sample_list.insert(insert_position, sample);
            }
            DestinationOrderQosPolicyKind::ByReceptionTimestamp => self.sample_list.push(sample),
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
        Ok(AddChangeResult::Added)
    }

    pub fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect()
    }

    pub fn read(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: &Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.create_sample_collection(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
            false,
        )
    }

    pub fn take(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: &Option<InstanceHandle>,
    ) -> DdsResult<SampleList> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.create_sample_collection(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
            true,
        )
    }
}
