use crate::infrastructure::{
    error::{DdsError, DdsResult},
    qos_policy::{HistoryQosPolicy, HistoryQosPolicyKind, Length, ResourceLimitsQosPolicy},
};

use super::{
    behavior_types::InstanceHandle,
    messages::{
        self,
        submessage_elements::{Data, ParameterList},
        submessages::data::DataSubmessage,
    },
    types::{ChangeKind, EntityId, Guid, SequenceNumber},
};
use std::collections::{HashMap, VecDeque};

pub struct RtpsWriterCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    timestamp: messages::types::Time,
    data_value: Data,
    inline_qos: ParameterList,
}

impl RtpsWriterCacheChange {
    pub fn as_data_submessage(&self, reader_id: EntityId) -> DataSubmessage {
        let (data_flag, key_flag) = match self.kind() {
            ChangeKind::Alive => (true, false),
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (false, true),
            _ => todo!(),
        };

        DataSubmessage::new(
            true,
            data_flag,
            key_flag,
            false,
            reader_id,
            self.writer_guid().entity_id(),
            self.sequence_number(),
            self.inline_qos.clone(),
            self.data_value.clone(),
        )
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }
}

impl RtpsWriterCacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        timestamp: messages::types::Time,
        data_value: Data,
        inline_qos: ParameterList,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            instance_handle,
            timestamp,
            data_value,
            inline_qos,
        }
    }
}

impl RtpsWriterCacheChange {
    pub fn kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn writer_guid(&self) -> Guid {
        self.writer_guid
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn timestamp(&self) -> messages::types::Time {
        self.timestamp
    }

    pub fn data_value(&self) -> &Data {
        &self.data_value
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }
}

#[derive(Default)]
pub struct WriterHistoryCache {
    changes: HashMap<InstanceHandle, VecDeque<RtpsWriterCacheChange>>,
    max_seq_num: Option<SequenceNumber>,
}

impl WriterHistoryCache {
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
            max_seq_num: None,
        }
    }

    pub fn change_list(&self) -> impl Iterator<Item = &RtpsWriterCacheChange> {
        self.changes.values().flatten()
    }

    pub fn add_change(
        &mut self,
        change: RtpsWriterCacheChange,
        history_qos_policy: &HistoryQosPolicy,
        resource_limits_qos_policy: &ResourceLimitsQosPolicy,
    ) -> DdsResult<()> {
        if let Length::Limited(max_instances) = resource_limits_qos_policy.max_instances {
            if !self.changes.contains_key(&change.instance_handle)
                && self.changes.len() == max_instances as usize
            {
                return Err(DdsError::OutOfResources);
            }
        }

        self.changes.entry(change.instance_handle()).or_default();

        if let HistoryQosPolicyKind::KeepLast(depth) = history_qos_policy.kind {
            if self.changes[&change.instance_handle()].len() == depth as usize {
                self.changes
                    .get_mut(&change.instance_handle())
                    .expect("InstanceHandle entry must exist")
                    .pop_front();
            }
        }

        // Only Alive changes count towards the resource limits
        if let Length::Limited(max_samples_per_instance) =
            resource_limits_qos_policy.max_samples_per_instance
        {
            if change.kind == ChangeKind::Alive
                && self.changes[&change.instance_handle()].len()
                    == max_samples_per_instance as usize
            {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_samples) = resource_limits_qos_policy.max_samples {
            let total_samples = self.changes.iter().fold(0, |acc, (_, s)| {
                let total_instance_samples =
                    s.iter().filter(|cc| cc.kind == ChangeKind::Alive).count();
                acc + total_instance_samples
            });
            if total_samples == max_samples as usize {
                return Err(DdsError::OutOfResources);
            }
        }

        if change.sequence_number() > self.max_seq_num.unwrap_or(0) {
            self.max_seq_num = Some(change.sequence_number())
        }

        self.changes
            .get_mut(&change.instance_handle())
            .expect("InstanceHandle entry must exist")
            .push_back(change);

        Ok(())
    }

    pub fn remove_change<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        for changes_of_instance in self.changes.values_mut() {
            changes_of_instance.retain(|cc| !f(cc));
        }
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes
            .values()
            .flatten()
            .map(|cc| cc.sequence_number)
            .min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.max_seq_num
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::types::GUID_UNKNOWN;
    use tests::messages::types::TIME_INVALID;

    const HANDLE_NIL: InstanceHandle = InstanceHandle([0; 16]);

    #[test]
    fn remove_change() {
        let mut hc = WriterHistoryCache::new();
        let change = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            InstanceHandle([0; 16]),
            1,
            TIME_INVALID,
            Data::default(),
            ParameterList::empty(),
        );
        hc.add_change(
            change,
            &HistoryQosPolicy::default(),
            &ResourceLimitsQosPolicy::default(),
        )
        .unwrap();
        hc.remove_change(|cc| cc.sequence_number() == 1);
        assert!(hc.change_list().count() == 0);
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = WriterHistoryCache::new();
        let change1 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            InstanceHandle([0; 16]),
            1,
            TIME_INVALID,
            Data::default(),
            ParameterList::empty(),
        );
        let change2 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            2,
            TIME_INVALID,
            Data::default(),
            ParameterList::empty(),
        );
        hc.add_change(
            change1,
            &HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
            },
            &ResourceLimitsQosPolicy::default(),
        )
        .unwrap();
        hc.add_change(
            change2,
            &HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
            },
            &ResourceLimitsQosPolicy::default(),
        )
        .unwrap();
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc = WriterHistoryCache::new();
        let change1 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            1,
            TIME_INVALID,
            Data::default(),
            ParameterList::empty(),
        );
        let change2 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            2,
            TIME_INVALID,
            Data::default(),
            ParameterList::empty(),
        );
        hc.add_change(
            change1,
            &HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
            },
            &ResourceLimitsQosPolicy::default(),
        )
        .unwrap();
        hc.add_change(
            change2,
            &HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
            },
            &ResourceLimitsQosPolicy::default(),
        )
        .unwrap();
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
