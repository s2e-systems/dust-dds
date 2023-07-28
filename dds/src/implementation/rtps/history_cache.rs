use std::collections::HashMap;

use super::{
    messages::{
        submessage_elements::{Data, ParameterList},
        submessages::{data::DataSubmessageWrite, data_frag::DataFragSubmessageWrite},
    },
    types::{ChangeKind, EntityId, Guid, SequenceNumber},
};
use crate::{
    implementation::rtps::messages::types::FragmentNumber,
    infrastructure::{
        instance::InstanceHandle,
        qos_policy::{HistoryQosPolicy, HistoryQosPolicyKind},
        time::Time,
    },
};

pub struct RtpsWriterCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    _instance_handle: InstanceHandle,
    timestamp: Time,
    data_value: Vec<Data>,
    inline_qos: ParameterList,
}

pub struct DataFragSubmessages<'a> {
    cache_change: &'a RtpsWriterCacheChange,
    reader_id: EntityId,
}

impl<'a> DataFragSubmessages<'a> {
    pub fn new(cache_change: &'a RtpsWriterCacheChange, reader_id: EntityId) -> Self {
        Self {
            cache_change,
            reader_id,
        }
    }
}

pub struct DataFragSubmessagesIter<'a> {
    cache_change: &'a RtpsWriterCacheChange,
    data: Vec<&'a Data>,
    reader_id: EntityId,
    pos: usize,
}

impl<'a> IntoIterator for &'a DataFragSubmessages<'a> {
    type Item = DataFragSubmessageWrite<'a>;
    type IntoIter = DataFragSubmessagesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let data = self.cache_change.data_value.iter().collect();
        Self::IntoIter {
            cache_change: self.cache_change,
            data,
            reader_id: self.reader_id,
            pos: 0,
        }
    }
}

impl<'a> Iterator for DataFragSubmessagesIter<'a> {
    type Item = DataFragSubmessageWrite<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.data.len() {
            let inline_qos_flag = true;
            let key_flag = match self.cache_change.kind() {
                ChangeKind::Alive => false,
                ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => true,
                _ => todo!(),
            };
            let non_standard_payload_flag = false;
            let reader_id = self.reader_id;
            let writer_id = self.cache_change.writer_guid().entity_id();
            let writer_sn = self.cache_change.sequence_number();
            let fragment_starting_num = FragmentNumber::new(self.pos as u32 + 1);
            let fragments_in_submessage = 1;
            let data_size = self.data.iter().map(|d| d.len()).sum::<usize>() as u32;
            let fragment_size = self.data[0].len() as u16;
            let inline_qos = &self.cache_change.inline_qos;
            let serialized_payload = self.data[self.pos];

            self.pos += 1;

            Some(DataFragSubmessageWrite::new(
                inline_qos_flag,
                non_standard_payload_flag,
                key_flag,
                reader_id,
                writer_id,
                writer_sn,
                fragment_starting_num,
                fragments_in_submessage,
                data_size,
                fragment_size,
                inline_qos,
                serialized_payload,
            ))
        } else {
            None
        }
    }
}

impl RtpsWriterCacheChange {
    pub fn as_data_submessage(&self, reader_id: EntityId) -> DataSubmessageWrite {
        let (data_flag, key_flag) = match self.kind() {
            ChangeKind::Alive => (true, false),
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (false, true),
            _ => todo!(),
        };

        DataSubmessageWrite::new(
            true,
            data_flag,
            key_flag,
            false,
            reader_id,
            self.writer_guid().entity_id(),
            self.sequence_number(),
            &self.inline_qos,
            &self.data_value[0],
        )
    }
}

impl RtpsWriterCacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        timestamp: Time,
        data_value: Vec<Data>,
        inline_qos: ParameterList,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            _instance_handle: instance_handle,
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

    pub fn _instance_handle(&self) -> InstanceHandle {
        self._instance_handle
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn timestamp(&self) -> Time {
        self.timestamp
    }

    pub fn data_value(&self) -> &[Data] {
        &self.data_value
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }
}

#[derive(Default)]
pub struct WriterHistoryCache {
    changes: HashMap<InstanceHandle, Vec<RtpsWriterCacheChange>>,
    history: HistoryQosPolicy,
}

impl WriterHistoryCache {
    pub fn new(history: HistoryQosPolicy) -> Self {
        Self {
            changes: HashMap::new(),
            history,
        }
    }

    pub fn change_list(&self) -> impl Iterator<Item=&RtpsWriterCacheChange> {
        self.changes.values().flatten()
    }

    pub fn add_change(&mut self, change: RtpsWriterCacheChange) {
        match self.history.kind {
            HistoryQosPolicyKind::KeepLast(depth) => {
                for changes_of_instance in self.changes.values_mut() {
                    changes_of_instance.truncate(depth as usize);
                }
            }
            HistoryQosPolicyKind::KeepAll => (),
        }
        let values = self.changes.entry(change._instance_handle()).or_default();
        values.push(change);
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
        self.changes.values().flatten().map(|cc| cc.sequence_number).min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes.values().flatten().map(|cc| cc.sequence_number).max()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        implementation::rtps::types::GUID_UNKNOWN,
        infrastructure::{instance::HANDLE_NIL, time::TIME_INVALID},
    };

    use super::*;

    #[test]
    fn remove_change() {
        let mut hc = WriterHistoryCache::new(HistoryQosPolicy::default());
        let change = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(1),
            TIME_INVALID,
            vec![Data::new(vec![])],
            ParameterList::empty(),
        );
        hc.add_change(change);
        hc.remove_change(|cc| cc.sequence_number() == SequenceNumber::new(1));
        assert!(hc.change_list().next().is_none());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = WriterHistoryCache::new(HistoryQosPolicy::default());
        let change1 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(1),
            TIME_INVALID,
            vec![Data::new(vec![])],
            ParameterList::empty(),
        );
        let change2 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(2),
            TIME_INVALID,
            vec![Data::new(vec![])],
            ParameterList::empty(),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(SequenceNumber::new(1)));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc = WriterHistoryCache::new(HistoryQosPolicy::default());
        let change1 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(1),
            TIME_INVALID,
            vec![Data::new(vec![])],
            ParameterList::empty(),
        );
        let change2 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(2),
            TIME_INVALID,
            vec![Data::new(vec![])],
            ParameterList::empty(),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(SequenceNumber::new(2)));
    }
}
