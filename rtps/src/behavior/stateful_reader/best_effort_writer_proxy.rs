use crate::types::{GuidPrefix, GUID};
use crate::behavior::WriterProxy;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::{Data, Gap};
use crate::behavior::cache_change_from_data;

use rust_dds_interface::history_cache::HistoryCache;

pub struct BestEffortWriterProxy(WriterProxy);

impl BestEffortWriterProxy {
    pub fn new(writer_proxy: WriterProxy) -> Self {
        Self(writer_proxy)
    }

    pub fn try_process_message(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>, history_cache: &mut HistoryCache) {
        self.waiting_state(src_guid_prefix, submessage, history_cache);
    }

    fn waiting_state(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>, history_cache: &mut HistoryCache) {
        if let Some(inner_submessage) = submessage {
            if self.is_submessage_destination(src_guid_prefix, inner_submessage) {
                match submessage.take().unwrap() {
                    RtpsSubmessage::Data(data) => self.transition_t2(history_cache, data),
                    RtpsSubmessage::Gap(gap) => self.transition_t4(gap),
                    _ => panic!("Unexpected reader message received"),
                }
            }
        }
    }

    fn transition_t2(&mut self, history_cache: &mut HistoryCache, data: Data) {
        let expected_seq_number = self.available_changes_max() + 1;
        if data.writer_sn() >= expected_seq_number {
            self.received_change_set(data.writer_sn());
            self.lost_changes_update(data.writer_sn());
            let cache_change = cache_change_from_data(data, &self.remote_writer_guid.prefix());
            history_cache.add_change(cache_change).unwrap();
        }
    }

    fn transition_t4(&mut self, gap: Gap) {
        for seq_num in gap.gap_start() .. gap.gap_list().base() - 1 {
            self.irrelevant_change_set(seq_num);
        }

        for &seq_num in gap.gap_list().set() {
            self.irrelevant_change_set(seq_num);
        }
    }

    fn is_submessage_destination(&self, src_guid_prefix: GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        let writer_id = match submessage {
            RtpsSubmessage::Data(data) => data.writer_id(),
            RtpsSubmessage::Gap(gap) => gap.writer_id(),
            _ => return false,
        };

        let writer_guid = GUID::new(src_guid_prefix, writer_id);
        if self.remote_writer_guid == writer_guid{
            true
        } else {
            false
        }
    }
}

impl std::ops::Deref for BestEffortWriterProxy {
    type Target = WriterProxy;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BestEffortWriterProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::{ChangeKind, GUID};
    // use crate::types::constants::{
    //     ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, LOCATOR_INVALID};
    // use crate::structure::{CacheChange, HistoryCacheResourceLimits};
    // use crate::messages::submessages::data_submessage::Payload;
    // use crate::serialized_payload::ParameterList;
    // use crate::inline_qos_types::KeyHash;
    // use crate::messages::Endianness;
    // use crate::behavior::change_kind_to_status_info;

    // #[test]
    // fn run_best_effort_data_only() {
    //     let mut history_cache = HistoryCache::new(HistoryCacheResourceLimits::default());
    //     let remote_writer_guid_prefix = [1;12];
    //     let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
    //     let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

    //     let mut best_effort_proxy = BestEffortWriterProxy::new(writer_proxy);

    //     let mut inline_qos = ParameterList::new();
    //     inline_qos.push(change_kind_to_status_info(ChangeKind::Alive));
    //     inline_qos.push(KeyHash([1;16]));

    //     let data1 = Data::new(
    //         Endianness::LittleEndian,
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, 
    //         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, 
    //         3,
    //         Some(inline_qos),
    //         Payload::Data(vec![1,2,3]));

    //     best_effort_proxy.input_queue.push_back(RtpsSubmessage::Data(data1));
    //     best_effort_proxy.process(&mut history_cache);

    //     let expected_change_1 = CacheChange::new(
    //         ChangeKind::Alive,
    //         remote_writer_guid,
    //         [1;16],
    //         3,
    //         Some(vec![1,2,3]),
    //         None,
    //     );

    //     assert_eq!(history_cache.changes().len(), 1);
    //     assert!(history_cache.changes().contains(&expected_change_1));
    //     assert_eq!(best_effort_proxy.writer_proxy.available_changes_max(), 3);

    //     // Run waiting state without any received message and verify nothing changes
    //     best_effort_proxy.process(&mut history_cache);
    //     assert_eq!(history_cache.changes().len(), 1);
    //     assert_eq!(best_effort_proxy.writer_proxy.available_changes_max(), 3);
    // }
}