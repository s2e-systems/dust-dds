use std::convert::TryInto;
use std::time::Instant;

use crate::types::{GuidPrefix, GUID, EntityId};
use crate::behavior::WriterProxy;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::{AckNack, Data, Gap, Heartbeat,};
use crate::messages::types::Count;

use crate::behavior::types::Duration;
use crate::behavior::{cache_change_from_data, BEHAVIOR_ENDIANNESS};

use rust_dds_interface::history_cache::HistoryCache;

pub struct ReliableWriterProxy {
    writer_proxy: WriterProxy,

    must_send_ack: bool,
    time_heartbeat_received: Instant,
    ackanck_count: Count,
    highest_received_heartbeat_count: Count,
}

impl ReliableWriterProxy {
    pub fn new(writer_proxy: WriterProxy) -> Self {
        Self {
            writer_proxy,
            must_send_ack: false,
            time_heartbeat_received: Instant::now(),
            ackanck_count: 0,
            highest_received_heartbeat_count: 0,
        }
    }

    pub fn try_process_message(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>, history_cache: &mut HistoryCache) {
        if let Some(inner_submessage) = submessage {
            if self.is_submessage_destination(src_guid_prefix, inner_submessage) {
                // If Waiting state (marked by the must_send_ack flag)
                if !self.must_send_ack {
                    self.waiting_heartbeat_state(inner_submessage);
                }

                self.ready_state(submessage, history_cache);
            }
        }
    }

    pub fn produce_messages(&mut self, reader_entity_id: EntityId, heartbeat_response_delay: Duration) -> Vec<RtpsSubmessage> {
        let mut output_queue = Vec::new();
        if self.must_send_ack {
            self.must_send_ack_state(reader_entity_id, heartbeat_response_delay, &mut output_queue);
        }
        output_queue
    }

    fn ready_state(&mut self, submessage: &mut Option<RtpsSubmessage>, history_cache: &mut HistoryCache) {
        match submessage.take().unwrap() {
            RtpsSubmessage::Data(data) => self.transition_t8(history_cache, data),
            RtpsSubmessage::Gap(gap) => self.transition_t9(gap),
            RtpsSubmessage::Heartbeat(heartbeat) => self.transition_t7(heartbeat),
            _ => panic!("Unexpected reader message received"),
        }
    }

    fn is_submessage_destination(&self, src_guid_prefix: GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        let writer_id = match submessage {
            RtpsSubmessage::Data(data) => data.writer_id(),
            RtpsSubmessage::Gap(gap) => gap.writer_id(),
            RtpsSubmessage::Heartbeat(heartbeat) => heartbeat.writer_id(),
            _ => return false,
        };

        let writer_guid = GUID::new(src_guid_prefix, writer_id);
        if self.remote_writer_guid == writer_guid{
            true
        } else {
            false
        }
    }

    fn transition_t8(&mut self, history_cache: &mut HistoryCache, data: Data) {
        let expected_seq_number = self.available_changes_max() + 1;
        if data.writer_sn() >= expected_seq_number {
            self.received_change_set(data.writer_sn());
            let cache_change = cache_change_from_data(data, &self.remote_writer_guid.prefix());
            history_cache.add_change(cache_change).unwrap();
            
        }
    }

    fn transition_t9(&mut self, gap: Gap) {
        for seq_num in gap.gap_start() .. gap.gap_list().base() - 1 {
            self.irrelevant_change_set(seq_num);
        }

        for &seq_num in gap.gap_list().set() {
            self.irrelevant_change_set(seq_num);
        }
    }

    fn transition_t7(&mut self, heartbeat: Heartbeat) {
        self.missing_changes_update(heartbeat.last_sn());
        self.lost_changes_update(heartbeat.first_sn());
    }

    fn waiting_heartbeat_state(&mut self, submessage: &RtpsSubmessage) {
        if let RtpsSubmessage::Heartbeat(heartbeat) = submessage {
            if !heartbeat.is_final() || 
                (heartbeat.is_final() && !self.missing_changes().is_empty()) {
                    self.time_heartbeat_received  = Instant::now();
                    self.must_send_ack = true;
            } 
        }   
    }

    fn must_send_ack_state(&mut self, reader_entity_id: EntityId, heartbeat_response_delay: Duration, output_queue: &mut Vec<RtpsSubmessage>) {
        let duration_since_heartbeat_received : Duration = self.time_heartbeat_received.elapsed().try_into().unwrap();
        if duration_since_heartbeat_received >  heartbeat_response_delay {
            self.transition_t5(reader_entity_id, output_queue)
        }
    }

    fn transition_t5(&mut self, reader_entity_id: EntityId, output_queue: &mut Vec<RtpsSubmessage>) {
        self.must_send_ack = false;
 
        self.ackanck_count += 1;
        let acknack = AckNack::new(
            BEHAVIOR_ENDIANNESS,
            reader_entity_id, 
            self.remote_writer_guid.entity_id(),
            self.available_changes_max(),
            self.missing_changes().clone(),
            self.ackanck_count,
            true);

        output_queue.push(RtpsSubmessage::AckNack(acknack));
    }
}

impl std::ops::Deref for ReliableWriterProxy {
    type Target = WriterProxy;

    fn deref(&self) -> &Self::Target {
        &self.writer_proxy
    }
}

impl std::ops::DerefMut for ReliableWriterProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer_proxy
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::types::{ChangeKind, GUID};
//     use crate::types::constants::{
//         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, LOCATOR_INVALID};
//     use crate::structure::{CacheChange, HistoryCacheResourceLimits};
//     use crate::messages::submessages::data_submessage::Payload;
//     use crate::serialized_payload::ParameterList;
//     use crate::inline_qos_types::KeyHash;
//     use crate::messages::Endianness;
//     use crate::behavior::change_kind_to_status_info;

//     #[test]
//     fn run_reliable_data_only() {
//         let mut history_cache = HistoryCache::new(HistoryCacheResourceLimits::default());
//         let heartbeat_response_delay = Duration::from_millis(500);
//         let reader_entity_id = ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR;
        
//         let remote_writer_guid_prefix = [1;12];
//         let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//         let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
//         let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

//         let mut inline_qos = ParameterList::new();
//         inline_qos.push(change_kind_to_status_info(ChangeKind::Alive));
//         inline_qos.push(KeyHash([1;16]));

//         let data1 = Data::new(
//             Endianness::LittleEndian,
//             reader_entity_id, 
//             ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, 
//             3,
//             Some(inline_qos),
//             Payload::Data(vec![1,2,3]));
            
//         reliable_writer_proxy.input_queue.push_back(RtpsSubmessage::Data(data1));


//         reliable_writer_proxy.process(&mut history_cache, reader_entity_id, heartbeat_response_delay);

//         let expected_change_1 = CacheChange::new(
//             ChangeKind::Alive,
//             remote_writer_guid,
//             [1;16],
//             3,
//             Some(vec![1,2,3]),
//             None,
//         );

//         assert_eq!(history_cache.changes().len(), 1);
//         assert!(history_cache.changes().contains(&expected_change_1));
//         assert_eq!(reliable_writer_proxy.writer_proxy.available_changes_max(), 0);

//         // Run without any received message and verify nothing changes
//         reliable_writer_proxy.process(&mut history_cache, reader_entity_id, heartbeat_response_delay);
//         assert_eq!(history_cache.changes().len(), 1);
//         assert_eq!(reliable_writer_proxy.writer_proxy.available_changes_max(), 0);
//     }

//     #[test]
//     fn run_reliable_non_final_heartbeat() {
//         let mut history_cache = HistoryCache::new(HistoryCacheResourceLimits::default());
//         let heartbeat_response_delay = Duration::from_millis(500);
//         let reader_entity_id = ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR;
        
//         let remote_writer_guid_prefix = [1;12];
//         let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//         let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
//         let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

//         let heartbeat = Heartbeat::new(
//             Endianness::LittleEndian,
//             reader_entity_id,
//             remote_writer_guid.entity_id(),
//             3,
//             6,
//             1,
//             false,
//             false,
//         );
    
//         reliable_writer_proxy.input_queue.push_back(RtpsSubmessage::Heartbeat(heartbeat));

//         reliable_writer_proxy.process(&mut history_cache, reader_entity_id, heartbeat_response_delay);
//         assert_eq!(reliable_writer_proxy.writer_proxy.missing_changes(), [3, 4, 5, 6].iter().cloned().collect());
//         assert_eq!(reliable_writer_proxy.must_send_ack(), true);
//     }
    
//     #[test]
//     fn run_reliable_final_heartbeat_with_missing_changes() {
//         let mut history_cache = HistoryCache::new(HistoryCacheResourceLimits::default());
//         let heartbeat_response_delay = Duration::from_millis(300);
//         let reader_entity_id = ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR;
        
//         let remote_writer_guid_prefix = [1;12];
//         let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//         let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
//         let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

//         let heartbeat = Heartbeat::new(
//             Endianness::LittleEndian,
//             ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
//             remote_writer_guid.entity_id(),
//             2,
//             3,
//             1,
//             true,
//             false,
//         );
//         reliable_writer_proxy.input_queue.push_back(RtpsSubmessage::Heartbeat(heartbeat));
        

//         reliable_writer_proxy.process(&mut history_cache, reader_entity_id, heartbeat_response_delay);
//         assert_eq!(reliable_writer_proxy.writer_proxy.missing_changes(), [2, 3].iter().cloned().collect());
//         assert_eq!(reliable_writer_proxy.must_send_ack(), true);

//         std::thread::sleep(heartbeat_response_delay.into());

//         reliable_writer_proxy.process(&mut history_cache, reader_entity_id, heartbeat_response_delay);
//         assert_eq!(reliable_writer_proxy.must_send_ack(), false);

//         // TODO: Test that AckNack is sent after duration
//     }

//     #[test]
//     fn run_reliable_final_heartbeat_without_missing_changes() {
//         let mut history_cache = HistoryCache::new(HistoryCacheResourceLimits::default());
//         let heartbeat_response_delay = Duration::from_millis(500);
//         let reader_entity_id = ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR;
        
//         let remote_writer_guid_prefix = [1;12];
//         let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//         let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);
//         let mut reliable_writer_proxy = ReliableWriterProxy::new(writer_proxy);

//         let heartbeat = Heartbeat::new(
//             Endianness::LittleEndian,
//             ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
//             remote_writer_guid.entity_id(),
//             1,
//             0,
//             1,
//             true,
//             false,
//         );
//         reliable_writer_proxy.input_queue.push_back(RtpsSubmessage::Heartbeat(heartbeat));

//         reliable_writer_proxy.process(&mut history_cache, reader_entity_id, heartbeat_response_delay);
//         assert_eq!(reliable_writer_proxy.writer_proxy.missing_changes(), [].iter().cloned().collect());
//         assert_eq!(reliable_writer_proxy.must_send_ack, false);
//     }
// }