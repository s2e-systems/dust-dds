use std::cmp::Ordering;
use std::sync::{Mutex};
use std::collections::{HashSet,VecDeque};

use crate::types::{GUID,GuidPrefix, SequenceNumber, ParameterList, InstanceHandle, Time, EntityId, Parameter, ProtocolVersion, VendorId, ChangeKind};
use crate::types::{ENTITYID_UNKNOWN, ENTITY_KIND_WRITER_WITH_KEY};
use crate::parser::{RtpsMessage, SubMessageType, InfoTs, InfoSrc, Data, Payload, InlineQosParameter};

#[derive(Hash, Eq, Default)]
#[allow(dead_code)]
pub struct CacheChange {
    // kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data: Option<Vec<u8>>,
    inline_qos: Option<ParameterList>,
}

impl CacheChange {
    pub fn new(writer_guid: GUID, instance_handle: InstanceHandle, sequence_number: SequenceNumber, data: Option<Vec<u8>>, inline_qos: Option<ParameterList>) -> CacheChange {
        CacheChange {
            writer_guid,
            instance_handle,
            sequence_number,
            data,
            inline_qos,
        }
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
    changes: Mutex<HashSet<CacheChange>>,
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

    pub fn process_message(&self, message: RtpsMessage) {
        let (mut source_guid_prefix, mut source_vendor_id, mut source_protocol_version, mut submessages) = message.take(); 
        let mut message_timestamp : Option<Time> = None;
        
        while let Some(submessage) = submessages.pop_front() {
            match submessage {
                SubMessageType::InfoTsSubmessage(info_ts) => self.process_infots(info_ts, &mut message_timestamp),
                SubMessageType::DataSubmessage(data) => self.process_data(data, &source_guid_prefix),
                SubMessageType::InfoSrcSubmessage(info_src) => self.process_infosrc(info_src, &mut source_protocol_version, &mut source_vendor_id, &mut source_guid_prefix),
                _ => println!("Unimplemented message type"),
            };   
        }
    }
    
    fn process_infots(&self, info_ts: InfoTs, time: &mut Option<Time>) {
        println!("Processing time");
        *time = info_ts.take();
    }
    
    fn process_infosrc(&self, info_src: InfoSrc, protocol_version: &mut ProtocolVersion, vendor_id: &mut VendorId, guid_prefix: &mut GuidPrefix) {
        println!("Processing info source");
        let (new_protocol_version, new_vendor_id, new_guid_prefix)=info_src.take();
        *protocol_version = new_protocol_version;
        *vendor_id = new_vendor_id;
        *guid_prefix = new_guid_prefix;
    }
    
    fn process_data(&self, data: Data, source_guid_prefix: &GuidPrefix) {
        println!("Processing data");
        let (reader_id, writer_id, writer_sn, inline_qos, serialized_payload) = data.take();
        let writer_guid = GUID::new(*source_guid_prefix, writer_id);
        
        if let Payload::Data(data) = serialized_payload {
            if let Some(inline_qos_list) = inline_qos {
                let key_hash_parameter = inline_qos_list.iter().find(|&x| x.is_key_hash());
                if let Some(InlineQosParameter::KeyHash(instance_handle)) = key_hash_parameter {
                    let cache_change = CacheChange::new(writer_guid,*instance_handle,writer_sn,Some(data),None);
                    self.add_change(cache_change);
                }
            }
        } else if let Payload::Key(key) = serialized_payload {
            if let Some(inline_qos_list) = inline_qos {
                let status_info_parameter = inline_qos_list.iter().find(|&x| x.is_status_info());
                if let Some(InlineQosParameter::StatusInfo(status_info)) = status_info_parameter {
                    // TODO: Check the liveliness changes to the entity
                }
            }
        } else {
            // TODO: Either no payload or non standardized payload. In either case, not implemented yet
        }
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