use byteorder::WriteBytesExt;

use crate::{
    implementation::rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
    infrastructure::{instance::InstanceHandle, time::Time},
};

use super::{
    messages::{
        submessage_elements::SequenceNumberSet,
        submessages::{DataFragSubmessage, DataSubmessageWrite, GapSubmessage},
        types::{ParameterId, SerializedPayload},
    },
    types::{ChangeKind, EntityId, Guid, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RtpsParameter {
    parameter_id: ParameterId,
    value: Vec<u8>,
}

impl MappingWriteByteOrdered for RtpsParameter {
    fn mapping_write_byte_ordered<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), std::io::Error> {
        let padding: &[u8] = match self.value.len() % 4 {
            1 => &[0; 3],
            2 => &[0; 2],
            3 => &[0; 1],
            _ => &[],
        };
        let length = self.value.len(); // + padding.len();
        writer.write_u16::<B>(self.parameter_id.into())?;
        writer.write_i16::<B>(length as i16)?;
        writer.write_all(self.value.as_ref())?;

        writer.write_all(padding)?;
        Ok(())
    }
}
impl NumberOfBytes for RtpsParameter {
    fn number_of_bytes(&self) -> usize {
        4 /* parameter_id and length */ + self.value.len()
    }
}

impl RtpsParameter {
    pub fn new(parameter_id: ParameterId, value: Vec<u8>) -> Self {
        Self {
            parameter_id,
            value,
        }
    }

    pub fn parameter_id(&self) -> ParameterId {
        self.parameter_id
    }

    pub fn value(&self) -> &[u8] {
        self.value.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsParameterList {
    parameter: Vec<RtpsParameter>,
}

impl RtpsParameterList {
    pub fn new(parameter: Vec<RtpsParameter>) -> Self {
        Self { parameter }
    }

    pub fn empty() -> Self {
        Self { parameter: vec![] }
    }

    pub fn parameter(&self) -> &[RtpsParameter] {
        self.parameter.as_ref()
    }
}
impl NumberOfBytes for RtpsParameterList {
    fn number_of_bytes(&self) -> usize {
        self.parameter.number_of_bytes() + 4 /* Sentinel */
    }
}

const PID_SENTINEL: u16 = 1;

impl MappingWriteByteOrdered for RtpsParameterList {
    fn mapping_write_byte_ordered<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), std::io::Error> {
        for parameter in self.parameter.iter() {
            parameter.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }
        PID_SENTINEL.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        [0u8; 2].mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

pub struct RtpsWriterCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    _instance_handle: InstanceHandle,
    timestamp: Time,
    data: Vec<u8>,
    inline_qos: RtpsParameterList,
}

impl RtpsWriterCacheChange {
    pub fn as_gap_message(&self, reader_id: EntityId) -> GapSubmessage {
        GapSubmessage {
            endianness_flag: true,
            reader_id,
            writer_id: self.writer_guid.entity_id(),
            gap_start: self.sequence_number,
            gap_list: SequenceNumberSet {
                base: self.sequence_number + 1,
                set: vec![],
            },
        }
    }

    pub fn as_data_submessage(&self, reader_id: EntityId) -> DataSubmessageWrite {
        let (data_flag, key_flag) = match self.kind() {
            ChangeKind::Alive => (true, false),
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (false, true),
            _ => todo!(),
        };

        DataSubmessageWrite {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag,
            key_flag,
            non_standard_payload_flag: false,
            reader_id,
            writer_id: self.writer_guid().entity_id(),
            writer_sn: self.sequence_number(),
            inline_qos: &self.inline_qos,
            serialized_payload: SerializedPayload::new(self.data_value()),
        }
    }

    pub fn as_data_frag_submessages(
        &self,
        max_bytes: usize,
        reader_id: EntityId,
    ) -> Vec<DataFragSubmessage> {
        // let data = self.data_value();
        // let data_size = ULong::new(data.len() as u32);
        // let mut fragment_starting_num = FragmentNumber::new(1);
        // const FRAGMENTS_IN_SUBMESSAGE: UShort = UShort::new(1);

        // let mut messages = Vec::new();

        // let mut data_fragment;
        // let mut data_remaining = data;

        // while !data_remaining.is_empty() {
        //     if data_remaining.len() >= max_bytes {
        //         (data_fragment, data_remaining) = data_remaining.split_at(max_bytes);
        //     } else {
        //         data_fragment = data_remaining;
        //         data_remaining = &[];
        //     }

        //     let endianness_flag = true;
        //     let inline_qos_flag = true;
        //     let key_flag = match self.kind() {
        //         ChangeKind::Alive => false,
        //         ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => true,
        //         _ => todo!(),
        //     };
        //     let non_standard_payload_flag = false;
        //     let writer_id = self.writer_guid().entity_id();
        //     let writer_sn = self.sequence_number();
        //     let inline_qos = ParameterList {
        //         parameter: self
        //             .inline_qos()
        //             .iter()
        //             .map(|p| Parameter {
        //                 parameter_id: p.parameter_id.0,
        //                 length: p.value.len() as i16,
        //                 value: p.value.as_ref(),
        //             })
        //             .collect(),
        //     };
        //     let serialized_payload = SerializedPayload::new(data_fragment);
        //     let message = DataFragSubmessage {
        //         endianness_flag,
        //         inline_qos_flag,
        //         non_standard_payload_flag,
        //         key_flag,
        //         reader_id,
        //         writer_id,
        //         writer_sn,
        //         fragment_starting_num,
        //         fragments_in_submessage: FRAGMENTS_IN_SUBMESSAGE,
        //         data_size,
        //         fragment_size: UShort::new(max_bytes as u16),
        //         inline_qos,
        //         serialized_payload,
        //     };

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
        data_value: Vec<u8>,
        inline_qos: RtpsParameterList,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            _instance_handle: instance_handle,
            timestamp,
            data: data_value,
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

    pub fn data_value(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn inline_qos(&self) -> &RtpsParameterList {
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
            vec![],
            RtpsParameterList::empty(),
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
            vec![],
            RtpsParameterList::empty(),
        );
        let change2 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(2),
            TIME_INVALID,
            vec![],
            RtpsParameterList::empty(),
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
            vec![],
            RtpsParameterList::empty(),
        );
        let change2 = RtpsWriterCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            SequenceNumber::new(2),
            TIME_INVALID,
            vec![],
            RtpsParameterList::empty(),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(SequenceNumber::new(2)));
    }
}
