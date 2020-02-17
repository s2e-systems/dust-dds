use crate::cache::{ReaderHistoryCache};
use crate::types::{LocatorList};
use std::collections::VecDeque;

use crate::parser::{RtpsMessage, InfoTs, Data, InfoSrc, SubMessageType, Payload, InlineQosParameter};
use crate::types::{Time, ProtocolVersion, VendorId, GuidPrefix, GUID, ChangeKind};
use crate::cache::{HistoryCache, CacheChange};

pub struct Receiver {
    // sourceVersion
    // sourceVendorId
    // sourceGuidPrefix
    // destGuidPrefix
    // UnicastReplyLocatorList
    // multicastReplyLocatorList
    // haveTimestamp
    // timestamp
    // messageLength
}

// impl Receiver
// {
    
// }

#[cfg(test)]
mod tests{

    use super::*;
    use crate::types::{EntityId};
    
    // #[test]
    // fn test_fill_history_cache(){
    //     let mut hc = ReaderHistoryCache::new();

    //     // Construct RTPS message with 1 info and 1 data submessage
    //     let mut rtps_message = RtpsMessage::new([0,1,2,3,4,5,6,7,8,9,10,11], [99,99], ProtocolVersion{major:2,minor:4});
    //     let time_submessage = SubMessageType::InfoTsSubmessage(InfoTs::new(Some(Time{seconds:10, fraction:1}))); 
    //     rtps_message.add_submessage(time_submessage);
    //     let data_submessage = SubMessageType::DataSubmessage(Data::new(
    //         EntityId::new([0,0,0],0),
    //         EntityId::new([0,1,0],1),
    //         1,
    //         Some(vec!(InlineQosParameter::KeyHash([0,1,2,3,4,5,6,7,8,9,10,11,0,1,0,1]))),
    //         Payload::Data(vec!(1)),
    //     ));
    //     rtps_message.add_submessage(data_submessage);

    //     let cache_changes = process_message(rtps_message, &mut hc);

    //     assert_eq!(hc.get_changes().lock().unwrap().len(), 1);

    //     let lock = hc.get_changes().lock().unwrap();
    //     let cache_change = lock.iter().next().unwrap().1;
    //     assert_eq!(cache_change.cache_change.get_change_kind(), &ChangeKind::Alive);
    //     assert_eq!(cache_change.cache_change.get_writer_guid(), &GUID::new([0,1,2,3,4,5,6,7,8,9,10,11], EntityId::new([0,1,0],1)));
    //     assert_eq!(cache_change.cache_change.get_instance_handle(), &[0,1,2,3,4,5,6,7,8,9,10,11,0,1,0,1]);
    //     assert_eq!(cache_change.cache_change.get_sequence_number(), &1);
    //     assert_eq!(cache_change.cache_change.get_inline_qos(), &None);
    // }



    // #[test]
    // fn test_process_info_ts_and_data_rtps_message() {
    //     let mut rtps_message = RtpsMessage::new([0,1,2,3,4,5,6,7,8,9,10,11], [99,99], ProtocolVersion{major:2,minor:4});

    //     let time_submessage = SubMessageType::InfoTsSubmessage(InfoTs::new(Some(Time{seconds:10, fraction:1}))); 
    //     rtps_message.add_submessage(time_submessage);

    //     let data_submessage = SubMessageType::DataSubmessage(Data::new(
    //         EntityId::new([0,0,0],0),
    //         EntityId::new([0,1,0],1),
    //         1,
    //         Some(vec!(InlineQosParameter::KeyHash([0,1,2,3,4,5,6,7,8,9,10,11,0,1,0,1]))),
    //         Payload::Data(vec!(1)),
    //     ));
    //     rtps_message.add_submessage(data_submessage);

    //     let cache_changes = process_message(rtps_message);
    //     assert_eq!(cache_changes.len(),1);
    //     assert_eq!(cache_changes[0].get_change_kind(), &ChangeKind::Alive);
    //     assert_eq!(cache_changes[0].get_writer_guid(), &GUID::new([0,1,2,3,4,5,6,7,8,9,10,11], EntityId::new([0,1,0],1)));
    //     assert_eq!(cache_changes[0].get_instance_handle(), &[0,1,2,3,4,5,6,7,8,9,10,11,0,1,0,1]);
    //     assert_eq!(cache_changes[0].get_sequence_number(), &1);
    //     assert_eq!(cache_changes[0].get_data(),&Some(vec!(1)));
    //     assert_eq!(cache_changes[0].get_inline_qos(), &None);
    // }

}
