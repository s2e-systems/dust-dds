use core::hash::Hash;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hasher;
use std::sync::{Mutex, MutexGuard};

use crate::messages::{
    Data, InfoSrc, InfoTs, InlineQosParameter, Payload, RtpsMessage, SubMessageType,
};
use crate::types::{
    ChangeKind, EntityId, GuidPrefix, InstanceHandle, ParameterList, ProtocolVersion,
    SequenceNumber, Time, VendorId, GUID,
};
use crate::types::{ENTITYID_UNKNOWN, ENTITY_KIND_WRITER_WITH_KEY};

#[derive(Eq, Debug, Clone)]
#[allow(dead_code)]
pub struct CacheChange {
    change_kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    inline_qos: Option<ParameterList<InlineQosParameter>>,
    data: Option<Vec<u8>>,
}

impl CacheChange {
    pub fn new(
        change_kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        inline_qos: Option<ParameterList<InlineQosParameter>>,
        data: Option<Vec<u8>>,
    ) -> CacheChange {
        CacheChange {
            change_kind,
            writer_guid,
            instance_handle,
            sequence_number,
            inline_qos,
            data,
        }
    }

    pub fn get_change_kind(&self) -> &ChangeKind {
        &self.change_kind
    }

    pub fn get_writer_guid(&self) -> &GUID {
        &self.writer_guid
    }

    pub fn get_instance_handle(&self) -> &InstanceHandle {
        &self.instance_handle
    }

    pub fn get_sequence_number(&self) -> &SequenceNumber {
        &self.sequence_number
    }

    pub fn get_inline_qos(&self) -> &Option<ParameterList<InlineQosParameter>> {
        &self.inline_qos
    }

    pub fn data(&self) -> Option<&Vec<u8>> {
        match &self.data {
            Some(data) => Some(data),
            None => None,
        }
    }

    pub fn clone_without_data(&self) -> Self {
        match *self {
            CacheChange {
                change_kind: ref __self_0_0,
                writer_guid: ref __self_0_1,
                instance_handle: ref __self_0_2,
                sequence_number: ref __self_0_3,
                inline_qos: ref __self_0_4,
                data: ref __self_0_5,
            } => {
        CacheChange {
            change_kind: *__self_0_0,
            writer_guid: * __self_0_1,
            instance_handle: * __self_0_2,
            sequence_number: * __self_0_3,
            inline_qos: __self_0_4.clone(),
            data: None,
        }}}
    }
}


impl PartialEq for CacheChange {
    fn eq(&self, other: &Self) -> bool {

        self.writer_guid == other.writer_guid
            && self.instance_handle == other.instance_handle
            && self.sequence_number == other.sequence_number
    }
}

impl Ord for CacheChange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence_number.cmp(&other.sequence_number)
    }
}

impl PartialOrd for CacheChange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sequence_number.cmp(&other.sequence_number))
    }
}

impl ::core::hash::Hash for CacheChange {
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
        match *self {
            CacheChange {
                change_kind: ref __self_0_0,
                writer_guid: ref __self_0_1,
                instance_handle: ref __self_0_2,
                sequence_number: ref __self_0_3,
                inline_qos: ref __self_0_4,
                data: ref __self_0_5,
            } => {
                ::core::hash::Hash::hash(&(*__self_0_0), state);
                ::core::hash::Hash::hash(&(*__self_0_1), state);
                ::core::hash::Hash::hash(&(*__self_0_2), state);
                ::core::hash::Hash::hash(&(*__self_0_3), state);
                ::core::hash::Hash::hash(&(*__self_0_4), state)
                // Explicitly ignore the data field
                // ::core::hash::Hash::hash(&(*__self_0_5), state)
            }
        }
    }
}

pub struct HistoryCache {
    changes: HashSet<CacheChange>,
}

impl HistoryCache {
    pub fn new() -> Self {
        HistoryCache {
            changes: HashSet::new(),
        }
    }

    pub fn add_change(&mut self, change: CacheChange) {
        self.changes.insert(change);
    }

    pub fn remove_change(&mut self, change: &CacheChange) {
        self.changes.remove(change);
    }

    pub fn get_changes(&self) -> &HashSet<CacheChange> {
        &self.changes
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        Some(self.changes.iter().min()?.sequence_number)
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        Some(self.changes.iter().max()?.sequence_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_change_list() {
        let mut history_cache = HistoryCache::new();
        let guid_prefix = [8; 12];
        let entity_id = EntityId::new([1, 2, 3], 4);
        let guid = GUID::new(guid_prefix, entity_id);
        let instance_handle = [9; 16];
        let sequence_number = 1;
        let data = Some(vec![4, 5, 6]);
        let cc = CacheChange::new(
            ChangeKind::Alive,
            guid,
            instance_handle,
            sequence_number,
            None,
            data,
        );
        let cc_clone_no_data = cc.clone_without_data();
        // cc_clone_no_data.data = None;
        let cc_clone = cc.clone();

        assert_eq!(history_cache.get_changes().len(), 0);
        history_cache.add_change(cc);
        assert_eq!(history_cache.get_changes().len(), 1);
        history_cache.add_change(cc_clone_no_data);
        assert_eq!(history_cache.get_changes().len(), 1);
        history_cache.remove_change(&cc_clone);
        assert_eq!(history_cache.get_changes().len(), 0);
    }

    #[test]
    fn cache_change_sequence_number() {
        let mut history_cache = HistoryCache::new();

        let guid_prefix = [8; 12];
        let entity_id = EntityId::new([1, 2, 3], 4);
        let guid = GUID::new(guid_prefix, entity_id);
        let instance_handle = [9; 16];
        let data = Some(vec![4, 5, 6]);
        let sequence_number_min = 1;
        let sequence_number_max = 2;
        let cc1 = CacheChange::new(
            ChangeKind::Alive,
            guid.clone(),
            instance_handle,
            sequence_number_min,
            None,
            data.clone(),
        );
        let cc2 = CacheChange::new(
            ChangeKind::Alive,
            guid.clone(),
            instance_handle,
            sequence_number_max,
            None,
            data.clone(),
        );

        assert_eq!(history_cache.get_seq_num_max(), None);
        history_cache.add_change(cc1);
        assert_eq!(
            history_cache.get_seq_num_min(),
            history_cache.get_seq_num_max()
        );
        history_cache.add_change(cc2);
        assert_eq!(history_cache.get_seq_num_min(), Some(sequence_number_min));
        assert_eq!(history_cache.get_seq_num_max(), Some(sequence_number_max));
    }

    #[test]
    fn cache_change_transport() {
        let data = [
            0x52, 0x54, 0x50, 0x53, 0x02, 0x01, 0x01, 0x02, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00,
            0x01, 0xbb, 0x00, 0x00, 0x00, 0x01, 0x09, 0x01, 0x08, 0x00, 0x9e, 0x81, 0xbc, 0x5d,
            0x97, 0xde, 0x48, 0x26, 0x15, 0x07, 0x1c, 0x01, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x70, 0x00, 0x10, 0x00, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x01, 0xc1, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
            0x15, 0x00, 0x04, 0x00, 0x02, 0x01, 0x00, 0x00, 0x16, 0x00, 0x04, 0x00, 0x01, 0x02,
            0x00, 0x00, 0x31, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf3, 0x1c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8,
            0x02, 0x04, 0x32, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf2, 0x1c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8,
            0x02, 0x04, 0x02, 0x00, 0x08, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x50, 0x00, 0x10, 0x00, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x01, 0xc1, 0x58, 0x00, 0x04, 0x00, 0x15, 0x04, 0x00, 0x00,
            0x00, 0x80, 0x04, 0x00, 0x15, 0x00, 0x00, 0x00, 0x07, 0x80, 0x5c, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x2f, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x50, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x44, 0x45, 0x53, 0x4b, 0x54, 0x4f,
            0x50, 0x2d, 0x4f, 0x52, 0x46, 0x44, 0x4f, 0x53, 0x35, 0x2f, 0x36, 0x2e, 0x31, 0x30,
            0x2e, 0x32, 0x2f, 0x63, 0x63, 0x36, 0x66, 0x62, 0x39, 0x61, 0x62, 0x33, 0x36, 0x2f,
            0x39, 0x30, 0x37, 0x65, 0x66, 0x66, 0x30, 0x32, 0x65, 0x33, 0x2f, 0x22, 0x78, 0x38,
            0x36, 0x5f, 0x36, 0x34, 0x2e, 0x77, 0x69, 0x6e, 0x2d, 0x76, 0x73, 0x32, 0x30, 0x31,
            0x35, 0x22, 0x2f, 0x00, 0x00, 0x00, 0x25, 0x80, 0x0c, 0x00, 0xd7, 0xf7, 0x20, 0x7f,
            0xbb, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        ];

        // let hc = ReaderHistoryCache::new();

        // let sender = std::net::UdpSocket::bind(SocketAddr::from((addr, 0))).unwrap();
        // sender.send_to(&data, SocketAddr::from((multicast_group, port))).unwrap();

        //aasert!(hc.changes.size == 1);
    }

    // #[test]
    // fn test_create_history_cache() {
    //     let empty_history_cache = ReaderHistoryCache::new();

    //     assert!(empty_history_cache.changes.read().unwrap().is_empty());
    // }

    // #[test]
    // fn test_add_and_remove_cache_change() {
    //     let history_cache = ReaderHistoryCache::new();
    //     assert_eq!(history_cache.changes.read().unwrap().len(), 0);

    //     let mut cache_change_sn1 = CacheChange::default();
    //     cache_change_sn1.instance_handle = [1;16];
    //     cache_change_sn1.sequence_number = 1;

    //     history_cache.add_change(cache_change_sn1).unwrap();

    //     assert_eq!(history_cache.changes.read().unwrap().len(), 1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap().len(), 1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[0].sequence_number, 1);

    //     let mut cache_change_sn2 = CacheChange::default();
    //     cache_change_sn2.instance_handle = [1;16];
    //     cache_change_sn2.sequence_number = 2;

    //     history_cache.add_change(cache_change_sn2).unwrap();
    //     assert_eq!(history_cache.changes.read().unwrap().len(), 1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap().len(), 2);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[0].sequence_number, 1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[1].sequence_number, 2);

    //     history_cache.remove_change(&[1;16], &1);
    //     assert_eq!(history_cache.changes.read().unwrap().len(), 1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap().len(), 1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[0].sequence_number, 2);

    //     history_cache.remove_instance(&[1;16]);

    //     assert_eq!(history_cache.changes.read().unwrap().len(), 0);
    // }

    // #[test]
    // fn test_process_info_ts_data_submessage() {
    //     let guid_prefix = [0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00, 0x00, 0x01,];
    //     let vendor_id = [0x01, 0x02];
    //     let protocol_version = ProtocolVersion{major: 0x02, minor: 0x01};
    //     let mut message = RtpsMessage::new(guid_prefix, vendor_id, protocol_version);

    //     let time_submessage = SubMessageType::InfoTsSubmessage(InfoTs::new(Some(Time{seconds: 1572635038, fraction: 642309783,})));

    //     let reader_id = ENTITYID_UNKNOWN;
    //     let writer_id = EntityId::new([1,2,3], ENTITY_KIND_WRITER_WITH_KEY);
    //     let writer_sn = 1; //SequenceNumber;
    //     let inline_qos = Some(vec!(Parameter{parameter_id: 0x0070, value: vec!(127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1, 0, 0, 1, 193) }));
    //     let serialized_payload = Payload::Data(vec!(1,2,3));
    //     let data_submessage = SubMessageType::DataSubmessage(
    //         Data::new(reader_id, writer_id, writer_sn, inline_qos, serialized_payload));

    //     message.add_submessage(time_submessage);
    //     message.add_submessage(data_submessage);

    //     let history_cache = ReaderHistoryCache::new();
    //     assert_eq!(history_cache.changes.read().unwrap().len(),0);

    //     history_cache.process_message(message);

    //     assert_eq!(history_cache.changes.read().unwrap().len(),1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap().len(), 1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[0].sequence_number, 1);
    // }
}
