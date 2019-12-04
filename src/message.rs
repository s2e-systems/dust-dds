use std::collections::VecDeque;

use crate::parser::{RtpsMessage, InfoTs, Data, InfoSrc, SubMessageType, Payload, InlineQosParameter};
use crate::types::{Time, ProtocolVersion, VendorId, GuidPrefix, GUID, ChangeKind};
use crate::cache::CacheChange;

pub fn process_message(message: RtpsMessage) -> VecDeque<CacheChange> {
    let (mut source_guid_prefix, mut source_vendor_id, mut source_protocol_version, mut submessages) = message.take(); 
    let mut message_timestamp : Option<Time> = None;
    
    let mut cache_changes = VecDeque::new();

    while let Some(submessage) = submessages.pop_front() {
        match submessage {
            SubMessageType::InfoTsSubmessage(info_ts) => process_infots(info_ts, &mut message_timestamp),
            SubMessageType::DataSubmessage(data) => process_data(data, &source_guid_prefix, &mut cache_changes),
            SubMessageType::InfoSrcSubmessage(info_src) => process_infosrc(info_src, &mut source_protocol_version, &mut source_vendor_id, &mut source_guid_prefix),
            _ => println!("Unimplemented message type"),
        };   
    }

    cache_changes
}

fn process_infots(info_ts: InfoTs, time: &mut Option<Time>) {
    *time = info_ts.take();
}

fn process_infosrc(info_src: InfoSrc, protocol_version: &mut ProtocolVersion, vendor_id: &mut VendorId, guid_prefix: &mut GuidPrefix) {
    let (new_protocol_version, new_vendor_id, new_guid_prefix)=info_src.take();
    *protocol_version = new_protocol_version;
    *vendor_id = new_vendor_id;
    *guid_prefix = new_guid_prefix;
}

fn process_data(data_submessage: Data, source_guid_prefix: &GuidPrefix, cache_changes: &mut VecDeque<CacheChange>) {
    let (reader_id, writer_id, writer_sn, inline_qos, serialized_payload) = data_submessage.take();
    let writer_guid = GUID::new(*source_guid_prefix, writer_id);
    
    if let Payload::Data(data) = serialized_payload {
        if let Some(inline_qos_list) = inline_qos {
            let key_hash_parameter = inline_qos_list.iter().find(|&x| x.is_key_hash());
            if let Some(InlineQosParameter::KeyHash(instance_handle)) = key_hash_parameter {
                let cache_change = CacheChange::new(ChangeKind::Alive, writer_guid,*instance_handle,writer_sn,Some(data),None);
                cache_changes.push_back(cache_change);
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

#[cfg(test)]
mod tests{

    use super::*;
    use crate::types::{EntityId};

    #[test]
    fn test_process_info_ts_and_data_rtps_message() {
        let mut rtps_message = RtpsMessage::new([0,1,2,3,4,5,6,7,8,9,10,11], [99,99], ProtocolVersion{major:2,minor:4});

        let time_submessage = SubMessageType::InfoTsSubmessage(InfoTs::new(Some(Time{seconds:10, fraction:1}))); 
        rtps_message.add_submessage(time_submessage);

        let data_submessage = SubMessageType::DataSubmessage(Data::new(
            EntityId::new([0,0,0],0),
            EntityId::new([0,1,0],1),
            1,
            Some(vec!(InlineQosParameter::KeyHash([0,1,2,3,4,5,6,7,8,9,10,11,0,1,0,1]))),
            Payload::Data(vec!(1)),
        ));
        rtps_message.add_submessage(data_submessage);

        let cache_changes = process_message(rtps_message);
        assert_eq!(cache_changes.len(),1);
        assert_eq!(cache_changes[0].get_change_kind(), &ChangeKind::Alive);
        assert_eq!(cache_changes[0].get_writer_guid(), &GUID::new([0,1,2,3,4,5,6,7,8,9,10,11], EntityId::new([0,1,0],1)));
        assert_eq!(cache_changes[0].get_instance_handle(), &[0,1,2,3,4,5,6,7,8,9,10,11,0,1,0,1]);
        assert_eq!(cache_changes[0].get_sequence_number(), &1);
        assert_eq!(cache_changes[0].get_data(),&Some(vec!(1)));
        assert_eq!(cache_changes[0].get_inline_qos(), &None);
    }

}
