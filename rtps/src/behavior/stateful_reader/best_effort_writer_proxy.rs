use std::collections::VecDeque;

use crate::types::{GuidPrefix, GUID};
use crate::structure::HistoryCache;
use crate::behavior::WriterProxy;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::{Data, Gap};

use crate::behavior::cache_change_from_data;
pub struct BestEffortWriterProxy {
    writer_proxy: WriterProxy,
    input_queue: VecDeque<RtpsSubmessage>,
}

impl BestEffortWriterProxy {
    pub fn new(writer_proxy: WriterProxy) -> Self {
        Self {
            writer_proxy,
            input_queue: VecDeque::new()
        }
    }

    pub fn process(&mut self, history_cache: &mut HistoryCache) {
        self.waiting_state(history_cache);
    }

    pub fn try_push_message(&mut self, _src_locator: crate::types::Locator, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        let writer_id = match submessage {
            Some(RtpsSubmessage::Data(data)) => data.writer_id(),
            Some(RtpsSubmessage::Gap(gap)) => gap.writer_id(),
            _ => return,
        };
        let writer_guid = GUID::new(src_guid_prefix, writer_id);

        if self.writer_proxy.remote_writer_guid() == &writer_guid {
            self.input_queue.push_back(submessage.take().unwrap())
        }
    }

    fn waiting_state(&mut self, history_cache: &mut HistoryCache) {
        let received = self.input_queue.pop_front();
        if let Some(received_message) = received  {
            match received_message {
                RtpsSubmessage::Data(data) => self.transition_t2(history_cache, data),
                RtpsSubmessage::Gap(gap) => self.transition_t4(&gap),
                RtpsSubmessage::Heartbeat(_) => (),
                _ => panic!("Unexpected reader message received"),
            }
        }
    }

    fn transition_t2(&mut self, history_cache: &mut HistoryCache, data: Data) {
        let expected_seq_number = self.writer_proxy.available_changes_max() + 1;
        if data.writer_sn() >= expected_seq_number {
            self.writer_proxy.received_change_set(data.writer_sn());
            self.writer_proxy.lost_changes_update(data.writer_sn());
            let cache_change = cache_change_from_data(data, &self.writer_proxy.remote_writer_guid().prefix());
            history_cache.add_change(cache_change).unwrap();
        }
    }

    fn transition_t4(&mut self, gap: &Gap) {
        for seq_num in gap.gap_start() .. gap.gap_list().base() - 1 {
            self.writer_proxy.irrelevant_change_set(seq_num);
        }

        for &seq_num in gap.gap_list().set() {
            self.writer_proxy.irrelevant_change_set(seq_num);
        }
    }

    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeKind, GUID};
    use crate::types::constants::{
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, LOCATOR_INVALID};
    use crate::structure::CacheChange;
    use crate::messages::submessages::data_submessage::Payload;
    use crate::serialized_payload::ParameterList;
    use crate::inline_qos_types::KeyHash;
    use crate::messages::Endianness;
    use crate::behavior::change_kind_to_status_info;

    #[test]
    fn run_best_effort_data_only() {
        let mut history_cache = HistoryCache::default();
        let remote_writer_guid_prefix = [1;12];
        let remote_writer_guid = GUID::new(remote_writer_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        let mut best_effort_proxy = BestEffortWriterProxy::new(writer_proxy);

        let mut inline_qos = ParameterList::new();
        inline_qos.push(change_kind_to_status_info(ChangeKind::Alive));
        inline_qos.push(KeyHash([1;16]));

        let data1 = Data::new(
            Endianness::LittleEndian,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, 
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, 
            3,
            Some(inline_qos),
            Payload::Data(vec![1,2,3]));

        best_effort_proxy.try_push_message(LOCATOR_INVALID,  remote_writer_guid_prefix, &mut Some(RtpsSubmessage::Data(data1)));
        best_effort_proxy.process(&mut history_cache);

        let expected_change_1 = CacheChange::new(
            ChangeKind::Alive,
            remote_writer_guid,
            [1;16],
            3,
            Some(vec![1,2,3]),
            None,
        );

        assert_eq!(history_cache.changes().len(), 1);
        assert!(history_cache.changes().contains(&expected_change_1));
        assert_eq!(best_effort_proxy.writer_proxy.available_changes_max(), 3);

        // Run waiting state without any received message and verify nothing changes
        best_effort_proxy.process(&mut history_cache);
        assert_eq!(history_cache.changes().len(), 1);
        assert_eq!(best_effort_proxy.writer_proxy.available_changes_max(), 3);
    }
}