use std::cmp::Ordering;
use std::sync::{Mutex};
use std::collections::{HashSet,VecDeque};

use crate::types::{GUID,GuidPrefix, SequenceNumber, ParameterList, InstanceHandle, Time, EntityId, Parameter, ProtocolVersion, VendorId, ChangeKind};
use crate::types::{ENTITYID_UNKNOWN, ENTITY_KIND_WRITER_WITH_KEY};
use crate::parser::{RtpsMessage, SubMessageType, InfoTs, InfoSrc, Data, Payload, InlineQosParameter};

#[derive(Hash, Eq, Debug)]
#[allow(dead_code)]
pub struct CacheChange {
    change_kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data: Option<Vec<u8>>,
    inline_qos: Option<ParameterList>,
}

impl CacheChange {
    pub fn new(change_kind: ChangeKind, writer_guid: GUID, instance_handle: InstanceHandle, sequence_number: SequenceNumber, data: Option<Vec<u8>>, inline_qos: Option<ParameterList>) -> CacheChange {
        CacheChange {
            change_kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data,
            inline_qos,
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

    pub fn get_data(&self) -> &Option<Vec<u8>> {
        &self.data
    }

    pub fn get_inline_qos(&self) -> &Option<ParameterList> {
        &self.inline_qos
    }
}

impl PartialEq for CacheChange {
    fn eq(&self, other: &Self) -> bool {
        self.writer_guid == other.writer_guid &&
        self.instance_handle == other.instance_handle &&
        self.sequence_number == other.sequence_number
    }
}

impl Ord for CacheChange
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence_number.cmp(&other.sequence_number)
    }
}

impl PartialOrd for CacheChange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sequence_number.cmp(&other.sequence_number))
    }
}

pub struct HistoryCache {
    pub changes: Mutex<HashSet<CacheChange>>,
}

impl HistoryCache {
    pub fn new() -> HistoryCache {
        HistoryCache {
            changes: Mutex::new(HashSet::with_capacity(1)),
        }
    }

    pub fn add_change(&self, change: CacheChange) -> Result<(),()> {
        self.changes.lock().unwrap().insert(change);

        Ok(())
    }
    
    pub fn remove_change(&self, key: &InstanceHandle, sequence_number: &SequenceNumber) {
        unimplemented!()
        // if self.has_key(&key) {
        //     let map_read_lock = &self.changes.read().unwrap()[key];
        //     let mut vector_lock = map_read_lock.lock().unwrap();
        //     vector_lock.retain(|x| x.sequence_number != *sequence_number);
        // }
    }
    
    pub fn get_change(&self, key: &InstanceHandle, sequence_number: &SequenceNumber) {
        unimplemented!()
    }

    pub fn remove_instance(&self, key: &InstanceHandle) {
        unimplemented!()
        // if self.has_key(&key) {
        //     let mut map_write_lock = self.changes.write().unwrap();
        //     map_write_lock.remove(key);
        // }
    }

    pub fn get_seq_num_min(&self, key: &InstanceHandle) -> Option<SequenceNumber>{
        unimplemented!()
        // Some(self.changes.read().unwrap()[key].lock().unwrap().iter().max()?.sequence_number)
    }

    pub fn get_seq_num_max(&self, key: &InstanceHandle) -> Option<SequenceNumber>{
        unimplemented!()
        // Some(self.changes.read().unwrap()[key].lock().unwrap().iter().min()?.sequence_number)
    }

    fn has_key(&self, key: &InstanceHandle) -> bool{
        unimplemented!()
        // self.changes.read().unwrap().contains_key(key)
    }
}


#[cfg(test)]
mod tests{
    use super::*;

    // #[test]
    // fn test_create_history_cache() {
    //     let empty_history_cache = HistoryCache::new();

    //     assert!(empty_history_cache.changes.read().unwrap().is_empty());
    // }

    // #[test]
    // fn test_add_and_remove_cache_change() {
    //     let history_cache = HistoryCache::new();
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

    //     let history_cache = HistoryCache::new();
    //     assert_eq!(history_cache.changes.read().unwrap().len(),0);

    //     history_cache.process_message(message);

    //     assert_eq!(history_cache.changes.read().unwrap().len(),1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap().len(), 1);
    //     assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[0].sequence_number, 1);
    // }
}