use std::collections::{HashSet};
use std::sync::{Mutex, MutexGuard};

use crate::types::{SequenceNumber, };

use super::cache_change::CacheChange;

pub struct HistoryCache {
    changes: Mutex<HashSet<CacheChange>>,
}

impl HistoryCache {
    pub fn new() -> Self {
        HistoryCache {
            changes: Mutex::new(HashSet::new()),
        }
    }

    pub fn add_change(&self, change: CacheChange) {
        self.changes().insert(change);
    }

    pub fn remove_change(&self, change: &CacheChange) {
        self.changes().remove(change);
    }

    pub fn changes(&self) -> MutexGuard<HashSet<CacheChange>> {
        self.changes.lock().unwrap()
    }

    // pub fn get_change_with_sequence_number(&self, sequence_number: &SequenceNumber) -> Option<&CacheChange> {
    //     self.changes.lock().unwrap().iter().find(|cc| cc.sequence_number() == sequence_number)
    // }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        Some(self.changes().iter().min()?.sequence_number())
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        Some(self.changes().iter().max()?.sequence_number())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntityId, EntityKind, GUID, ChangeKind};

    #[test]
    fn cache_change_list() {
        let history_cache = HistoryCache::new();
        let guid_prefix = [8; 12];
        let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);
        let guid = GUID::new(guid_prefix, entity_id);
        let instance_handle = [9; 16];
        let sequence_number = 1;
        let data_value = Some(vec![4, 5, 6]);
        let cc = CacheChange::new(
            ChangeKind::Alive,
            guid,
            instance_handle,
            sequence_number,
            data_value,
            None,
        );

        let cc_clone = cc.clone();

        assert_eq!(history_cache.changes().len(), 0);
        history_cache.add_change(cc);
        assert_eq!(history_cache.changes().len(), 1);
        history_cache.remove_change(&cc_clone);
        assert_eq!(history_cache.changes().len(), 0);
    }

    #[test]
    fn cache_change_sequence_number() {
        let history_cache = HistoryCache::new();

        let guid_prefix = [8; 12];
        let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);
        let guid = GUID::new(guid_prefix, entity_id);
        let instance_handle = [9; 16];
        let data_value = Some(vec![4, 5, 6]);
        let sequence_number_min = 1;
        let sequence_number_max = 2;
        let cc1 = CacheChange::new(
            ChangeKind::Alive,
            guid.clone(),
            instance_handle,
            sequence_number_min,
            data_value.clone(),
            None,
        );
        let cc2 = CacheChange::new(
            ChangeKind::Alive,
            guid.clone(),
            instance_handle,
            sequence_number_max,
            data_value.clone(),
            None,
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
        // let data_value = [
        //     0x52, 0x54, 0x50, 0x53, 0x02, 0x01, 0x01, 0x02, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00,
        //     0x01, 0xbb, 0x00, 0x00, 0x00, 0x01, 0x09, 0x01, 0x08, 0x00, 0x9e, 0x81, 0xbc, 0x5d,
        //     0x97, 0xde, 0x48, 0x26, 0x15, 0x07, 0x1c, 0x01, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x01, 0x00, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        //     0x70, 0x00, 0x10, 0x00, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00,
        //     0x00, 0x01, 0x00, 0x00, 0x01, 0xc1, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
        //     0x15, 0x00, 0x04, 0x00, 0x02, 0x01, 0x00, 0x00, 0x16, 0x00, 0x04, 0x00, 0x01, 0x02,
        //     0x00, 0x00, 0x31, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf3, 0x1c, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8,
        //     0x02, 0x04, 0x32, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf2, 0x1c, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8,
        //     0x02, 0x04, 0x02, 0x00, 0x08, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        //     0x50, 0x00, 0x10, 0x00, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00,
        //     0x00, 0x01, 0x00, 0x00, 0x01, 0xc1, 0x58, 0x00, 0x04, 0x00, 0x15, 0x04, 0x00, 0x00,
        //     0x00, 0x80, 0x04, 0x00, 0x15, 0x00, 0x00, 0x00, 0x07, 0x80, 0x5c, 0x00, 0x00, 0x00,
        //     0x00, 0x00, 0x2f, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        //     0x50, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x44, 0x45, 0x53, 0x4b, 0x54, 0x4f,
        //     0x50, 0x2d, 0x4f, 0x52, 0x46, 0x44, 0x4f, 0x53, 0x35, 0x2f, 0x36, 0x2e, 0x31, 0x30,
        //     0x2e, 0x32, 0x2f, 0x63, 0x63, 0x36, 0x66, 0x62, 0x39, 0x61, 0x62, 0x33, 0x36, 0x2f,
        //     0x39, 0x30, 0x37, 0x65, 0x66, 0x66, 0x30, 0x32, 0x65, 0x33, 0x2f, 0x22, 0x78, 0x38,
        //     0x36, 0x5f, 0x36, 0x34, 0x2e, 0x77, 0x69, 0x6e, 0x2d, 0x76, 0x73, 0x32, 0x30, 0x31,
        //     0x35, 0x22, 0x2f, 0x00, 0x00, 0x00, 0x25, 0x80, 0x0c, 0x00, 0xd7, 0xf7, 0x20, 0x7f,
        //     0xbb, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        // ];

        // let hc = ReaderHistoryCache::new();

        // let sender = std::net::UdpSocket::bind(SocketAddr::from((addr, 0))).unwrap();
        // sender.send_to(&data_value, SocketAddr::from((multicast_group, port))).unwrap();

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
