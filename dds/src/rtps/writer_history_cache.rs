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
    max_changes: Option<i32>,
}

impl WriterHistoryCache {
    pub fn new(max_changes: Option<i32>) -> Self {
        Self {
            changes: HashMap::new(),
            max_seq_num: None,
            max_changes,
        }
    }

    pub fn change_list(&self) -> impl Iterator<Item = &RtpsWriterCacheChange> {
        self.changes.values().flatten()
    }

    pub fn add_change(&mut self, change: RtpsWriterCacheChange) {
        let changes_of_instance = self.changes.entry(change.instance_handle()).or_default();

        if let Some(max_changes) = self.max_changes {
            if changes_of_instance.len() == max_changes as usize {
                changes_of_instance.pop_front();
            }
        }

        if change.sequence_number() > self.max_seq_num.unwrap_or(0) {
            self.max_seq_num = Some(change.sequence_number())
        }

        changes_of_instance.push_back(change);
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
        let mut hc = WriterHistoryCache::new(None);
        let change = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            InstanceHandle([0; 16]),
            1,
            TIME_INVALID,
            Data::default(),
            ParameterList::empty(),
        );
        hc.add_change(change);
        hc.remove_change(|cc| cc.sequence_number() == 1);
        assert!(hc.change_list().count() == 0);
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = WriterHistoryCache::new(None);
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
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc = WriterHistoryCache::new(None);
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
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
