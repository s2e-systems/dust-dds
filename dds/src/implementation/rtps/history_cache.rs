use std::marker::PhantomData;

use itertools::Itertools;

use crate::{
    implementation::rtps::messages::types::FragmentNumber,
    infrastructure::{instance::InstanceHandle, time::Time},
};

use super::{
    messages::{
        overall_structure::{FromBytes, WriteBytes},
        submessage_elements::{ParameterList, SequenceNumberSet},
        submessages::{
            data::DataSubmessageWrite, data_frag::DataFragSubmessageWrite, gap::GapSubmessageWrite,
        },
    },
    types::{ChangeKind, EntityId, Guid, SequenceNumber, ENTITYID_UNKNOWN},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Data(Vec<u8>);

impl Data {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn chunks(&self, size: usize) -> Vec<Data> {
        self.0.chunks(size).map(|c| Data(c.into())).collect()
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl WriteBytes for &Data {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        buf[..self.0.len()].copy_from_slice(&self.0);
        let length_inclusive_padding = (self.0.len() + 3) & !3;
        buf[self.0.len()..length_inclusive_padding].fill(0);
        length_inclusive_padding
    }
}

impl FromBytes<'_> for Data {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self {
        Self::new(v.to_vec())
    }
}

pub struct RtpsWriterCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    _instance_handle: InstanceHandle,
    timestamp: Time,
    data_value: Data,
    inline_qos: ParameterList,
}

pub struct RtpsWriterCacheChangeFrag<'a> {
    cache_change: &'a RtpsWriterCacheChange,
    reader_id: EntityId,
    data: Vec<Data>,
}

impl<'a> RtpsWriterCacheChangeFrag<'a> {
    pub fn new(cache_change: &'a RtpsWriterCacheChange, data_max_size_serialized: usize, reader_id: EntityId) -> Self {
        let data = cache_change.data_value.chunks(data_max_size_serialized);
        Self {
            cache_change,
            reader_id,
            data,
        }
    }
}

pub struct DataFragSubmessagesIter<'a> {
    cache_change: &'a RtpsWriterCacheChangeFrag<'a>,
    data: Vec<&'a Data>,
}

impl<'a> IntoIterator for &'a RtpsWriterCacheChangeFrag<'a> {
    type Item = DataFragSubmessageWrite<'a>;
    type IntoIter = DataFragSubmessagesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let data = self.data.iter().collect();
        Self::IntoIter {
            cache_change: self,
            data,
        }
    }
}

impl<'a> DataFragSubmessagesIter<'a> {
    pub fn new(cache_change: &'a RtpsWriterCacheChangeFrag<'a>) -> Self {
        let data = cache_change.data.iter().collect();
        Self { cache_change, data }
    }
}
impl<'a> Iterator for DataFragSubmessagesIter<'a> {
    type Item = DataFragSubmessageWrite<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let inline_qos_flag = true;
        let key_flag = match self.cache_change.cache_change.kind() {
            ChangeKind::Alive => false,
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => true,
            _ => todo!(),
        };
        let non_standard_payload_flag = false;
        let reader_id = self.cache_change.reader_id;
        let writer_id = self.cache_change.cache_change.writer_guid().entity_id();
        let writer_sn = self.cache_change.cache_change.sequence_number();
        let fragment_starting_num = FragmentNumber::new(1);
        let fragments_in_submessage = 1;
        let data_size = 1;
        let fragment_size = 1;
        let inline_qos = &self.cache_change.cache_change.inline_qos;
        let serialized_payload = self.data[0];
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
    }
}

pub struct DataFragSubmessages<'a> {
    cache_change: &'a RtpsWriterCacheChange,
    reader_id: EntityId,
    data: Data,
}

impl<'a> DataFragSubmessages<'a> {
    pub fn new(cache_change: &'a RtpsWriterCacheChange, reader_id: EntityId) -> Self {
        let data = cache_change.data_value().clone();
        Self {
            cache_change,
            reader_id,
            data,
        }
    }

    pub fn next<'b>(&'b mut self) -> Option<DataFragSubmessageWrite<'b>> {
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
        let fragment_starting_num = FragmentNumber::new(1);
        let fragments_in_submessage = 1;
        let data_size = 1;
        let fragment_size = 1;
        let inline_qos = &self.cache_change.inline_qos;
        let serialized_payload = &self.data;
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
    }
}

impl RtpsWriterCacheChange {
    pub fn as_gap_message(&self, reader_id: EntityId) -> GapSubmessageWrite {
        GapSubmessageWrite::new(
            reader_id,
            self.writer_guid.entity_id(),
            self.sequence_number,
            SequenceNumberSet {
                base: self.sequence_number + 1,
                set: vec![],
            },
        )
    }

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
            &self.data_value,
        )
    }

    pub fn as_data_frag_submessages(
        &self,
        max_bytes: usize,
        reader_id: EntityId,
    ) -> Vec<DataFragSubmessageWrite> {
        // let data_value = self.data_value().as_ref();
        // let data_size = data_value.len() as u32;
        // let mut fragment_starting_num = FragmentNumber::new(1);
        // const FRAGMENTS_IN_SUBMESSAGE: u16 = 1;

        // let mut messages = Vec::new();

        // let mut data_fragment;
        // let mut data_remaining = data_value;

        // while !data_remaining.is_empty() {
        //     if data_remaining.len() >= max_bytes {
        //         (data_fragment, data_remaining) = data_remaining.split_at(max_bytes);
        //     } else {
        //         data_fragment = data_remaining;
        //         data_remaining = &[];
        //     }

        //     let inline_qos_flag = true;
        //     let key_flag = match self.kind() {
        //         ChangeKind::Alive => false,
        //         ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => true,
        //         _ => todo!(),
        //     };
        //     let non_standard_payload_flag = false;
        //     let writer_id = self.writer_guid().entity_id();
        //     let writer_sn = self.sequence_number();
        //     let inline_qos = &self.inline_qos;
        //     let serialized_payload = &Data::new(data_fragment.to_vec());
        //     let message = DataFragSubmessageWrite::new(
        //         inline_qos_flag,
        //         non_standard_payload_flag,
        //         key_flag,
        //         reader_id,
        //         writer_id,
        //         writer_sn,
        //         fragment_starting_num,
        //         FRAGMENTS_IN_SUBMESSAGE,
        //         data_size,
        //         max_bytes as u16,
        //         inline_qos,
        //         serialized_payload,
        //     );

        //     messages.push(message);

        //     fragment_starting_num += FragmentNumber::new(1);
        // }
        // messages
        todo!()
    }
}

impl RtpsWriterCacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        timestamp: Time,
        data_value: Data,
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

    pub fn data_value(&self) -> &Data {
        &self.data_value
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }
}

#[derive(Default)]
pub struct WriterHistoryCache {
    changes: Vec<RtpsWriterCacheChange>,
}

impl WriterHistoryCache {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    pub fn change_list(&self) -> &[RtpsWriterCacheChange] {
        &self.changes
    }

    pub fn add_change(&mut self, change: RtpsWriterCacheChange) {
        self.changes.push(change);
    }

    pub fn remove_change<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.changes.retain(|cc| !f(cc));
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes.iter().map(|cc| cc.sequence_number).min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes.iter().map(|cc| cc.sequence_number).max()
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
        let mut hc = WriterHistoryCache::new();
        let change = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(1),
            TIME_INVALID,
            Data::new(vec![]),
            ParameterList::empty(),
        );
        hc.add_change(change);
        hc.remove_change(|cc| cc.sequence_number() == SequenceNumber::new(1));
        assert!(hc.change_list().is_empty());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = WriterHistoryCache::new();
        let change1 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(1),
            TIME_INVALID,
            Data::new(vec![]),
            ParameterList::empty(),
        );
        let change2 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(2),
            TIME_INVALID,
            Data::new(vec![]),
            ParameterList::empty(),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(SequenceNumber::new(1)));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc = WriterHistoryCache::new();
        let change1 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(1),
            TIME_INVALID,
            Data::new(vec![]),
            ParameterList::empty(),
        );
        let change2 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(2),
            TIME_INVALID,
            Data::new(vec![]),
            ParameterList::empty(),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(SequenceNumber::new(2)));
    }
}
