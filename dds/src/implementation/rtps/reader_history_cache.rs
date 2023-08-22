use std::collections::{HashMap, HashSet};

use crate::{
    infrastructure::{
        instance::InstanceHandle,
        qos::DataReaderQos,
        qos_policy::{DestinationOrderQosPolicyKind, HistoryQosPolicyKind},
        status::SampleRejectedStatusKind,
        time::{DurationKind, Time},
    },
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
};

use super::{
    messages::submessage_elements::{Data, ParameterList},
    types::{ChangeKind, Guid},
};

pub type RtpsReaderResult<T> = Result<T, RtpsReaderError>;

#[derive(Debug, PartialEq, Eq)]
pub enum RtpsReaderError {
    InvalidData(&'static str),
    Rejected(InstanceHandle, SampleRejectedStatusKind),
}

pub struct RtpsReaderCacheChange {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub instance_handle: InstanceHandle,
    pub data: Data,
    pub inline_qos: ParameterList,
    pub source_timestamp: Option<Time>,
    pub sample_state: SampleStateKind,
    pub disposed_generation_count: i32,
    pub no_writers_generation_count: i32,
    pub reception_timestamp: Time,
}

pub struct Instance {
    pub view_state: ViewStateKind,
    pub instance_state: InstanceStateKind,
    pub most_recent_disposed_generation_count: i32,
    pub most_recent_no_writers_generation_count: i32,
}

impl Instance {
    pub fn new() -> Self {
        Self {
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
        }
    }

    pub fn update_state(&mut self, change_kind: ChangeKind) {
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

    pub fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }
}

pub struct ReaderHistoryCache {
    pub changes: Vec<RtpsReaderCacheChange>,
    pub instances: HashMap<InstanceHandle, Instance>,
}

impl ReaderHistoryCache {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            instances: HashMap::new(),
        }
    }

    pub fn add_change(
        &mut self,
        mut change: RtpsReaderCacheChange,
        qos: &DataReaderQos,
    ) -> RtpsReaderResult<InstanceHandle> {
        if self.is_sample_of_interest_based_on_time(&change, qos) {
            if self.is_max_samples_limit_reached(&change, qos) {
                Err(RtpsReaderError::Rejected(
                    change.instance_handle,
                    SampleRejectedStatusKind::RejectedBySamplesLimit,
                ))
            } else if self.is_max_instances_limit_reached(&change, qos) {
                Err(RtpsReaderError::Rejected(
                    change.instance_handle,
                    SampleRejectedStatusKind::RejectedByInstancesLimit,
                ))
            } else if self.is_max_samples_per_instance_limit_reached(&change, qos) {
                Err(RtpsReaderError::Rejected(
                    change.instance_handle,
                    SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit,
                ))
            } else {
                let num_alive_samples_of_instance = self
                    .changes
                    .iter()
                    .filter(|cc| {
                        cc.instance_handle == change.instance_handle && cc.kind == ChangeKind::Alive
                    })
                    .count() as i32;

                if let HistoryQosPolicyKind::KeepLast(depth) = qos.history.kind {
                    if depth == num_alive_samples_of_instance {
                        let index_sample_to_remove = self
                            .changes
                            .iter()
                            .position(|cc| {
                                cc.instance_handle == change.instance_handle
                                    && cc.kind == ChangeKind::Alive
                            })
                            .expect("Samples must exist");
                        self.changes.remove(index_sample_to_remove);
                    }
                }

                let instance_entry = self
                    .instances
                    .entry(change.instance_handle)
                    .or_insert_with(Instance::new);

                instance_entry.update_state(change.kind);

                change.disposed_generation_count =
                    instance_entry.most_recent_disposed_generation_count;
                change.no_writers_generation_count =
                    instance_entry.most_recent_no_writers_generation_count;
                let change_instance_handle = change.instance_handle;
                self.changes.push(change);

                match qos.destination_order.kind {
                    DestinationOrderQosPolicyKind::BySourceTimestamp => {
                        self.changes.sort_by(|a, b| {
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
                        .changes
                        .sort_by(|a, b| a.reception_timestamp.cmp(&b.reception_timestamp)),
                }

                Ok(change_instance_handle)
            }
        } else {
            Ok(change.instance_handle)
        }
    }

    fn is_sample_of_interest_based_on_time(
        &self,
        change: &RtpsReaderCacheChange,
        qos: &DataReaderQos,
    ) -> bool {
        let closest_timestamp_before_received_sample = self
            .changes
            .iter()
            .filter(|cc| cc.instance_handle == change.instance_handle)
            .filter(|cc| cc.source_timestamp <= change.source_timestamp)
            .map(|cc| cc.source_timestamp)
            .max();

        if let Some(Some(t)) = closest_timestamp_before_received_sample {
            if let Some(sample_source_time) = change.source_timestamp {
                let sample_separation = sample_source_time - t;
                DurationKind::Finite(sample_separation) >= qos.time_based_filter.minimum_separation
            } else {
                true
            }
        } else {
            true
        }
    }

    fn is_max_samples_limit_reached(
        &self,
        change: &RtpsReaderCacheChange,
        qos: &DataReaderQos,
    ) -> bool {
        let total_samples = self
            .changes
            .iter()
            .filter(|cc| cc.instance_handle == change.instance_handle)
            .count();

        total_samples == qos.resource_limits.max_samples
    }

    fn is_max_instances_limit_reached(
        &self,
        change: &RtpsReaderCacheChange,
        qos: &DataReaderQos,
    ) -> bool {
        let instance_handle_list: HashSet<_> =
            self.changes.iter().map(|cc| cc.instance_handle).collect();

        if instance_handle_list.contains(&change.instance_handle) {
            false
        } else {
            instance_handle_list.len() == qos.resource_limits.max_instances
        }
    }

    fn is_max_samples_per_instance_limit_reached(
        &self,
        change: &RtpsReaderCacheChange,
        qos: &DataReaderQos,
    ) -> bool {
        let total_samples_of_instance = self
            .changes
            .iter()
            .filter(|cc| cc.instance_handle == change.instance_handle)
            .count();

        total_samples_of_instance == qos.resource_limits.max_samples_per_instance
    }
}
