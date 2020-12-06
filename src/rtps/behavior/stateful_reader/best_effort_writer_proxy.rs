use crate::rtps::behavior::cache_change_from_data;
use crate::rtps::behavior::WriterProxy;
use crate::rtps::messages::submessages::{Data, Gap};
use crate::rtps::messages::RtpsSubmessage;
use crate::rtps::types::{GuidPrefix, GUID};

use crate::rtps::structure::HistoryCache;

pub struct BestEffortWriterProxy(WriterProxy);

impl BestEffortWriterProxy {
    pub fn new(writer_proxy: WriterProxy) -> Self {
        Self(writer_proxy)
    }

    pub fn try_process_message(
        &mut self,
        src_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
        history_cache: &mut HistoryCache,
    ) {
        self.waiting_state(src_guid_prefix, submessage, history_cache);
    }

    fn waiting_state(
        &mut self,
        src_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
        history_cache: &mut HistoryCache,
    ) {
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
            history_cache.add_change(cache_change);
        }
    }

    fn transition_t4(&mut self, gap: Gap) {
        for seq_num in gap.gap_start()..gap.gap_list().base() - 1 {
            self.irrelevant_change_set(seq_num);
        }

        for &seq_num in gap.gap_list().set() {
            self.irrelevant_change_set(seq_num);
        }
    }

    fn is_submessage_destination(
        &self,
        src_guid_prefix: GuidPrefix,
        submessage: &RtpsSubmessage,
    ) -> bool {
        let writer_id = match submessage {
            RtpsSubmessage::Data(data) => data.writer_id(),
            RtpsSubmessage::Gap(gap) => gap.writer_id(),
            _ => return false,
        };

        let writer_guid = GUID::new(src_guid_prefix, writer_id);
        if self.remote_writer_guid == writer_guid {
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
    use super::*;
    use crate::rtps::behavior::change_kind_to_status_info;
    use crate::rtps::messages::submessages::data_submessage::Payload;
    use crate::rtps::messages::types::{Endianness, KeyHash};
    use crate::rtps::types::constants::{
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    };
    use crate::rtps::types::Locator;
    use crate::types::{ChangeKind, ParameterList};

    #[test]
    fn process_none_submessage() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut best_effort_writer_proxy = BestEffortWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let source_guid_prefix = [5; 12];
        let mut submessage = None;
        best_effort_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut submessage,
            &mut history_cache,
        );
    }

    #[test]
    fn process_data_submessage() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );
        let mut best_effort_writer_proxy = BestEffortWriterProxy::new(writer_proxy);

        let mut history_cache = HistoryCache::default();

        let source_guid_prefix = [5; 12];
        let status_info = change_kind_to_status_info(ChangeKind::Alive);
        let key_hash = KeyHash([1; 16]);
        let mut inline_qos = ParameterList::new();
        inline_qos.parameter.push(key_hash.into());
        inline_qos.parameter.push(status_info.into());
        let data_submessage = Data::new(
            Endianness::LittleEndian,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
            ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            1,
            Some(inline_qos),
            Payload::Data(vec![0, 1, 2]),
        );
        let expected_cache_change =
            cache_change_from_data(data_submessage.clone(), &source_guid_prefix);   

        best_effort_writer_proxy.try_process_message(
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Data(data_submessage)),
            &mut history_cache,
        );
        let received_change = history_cache.get_change(1).unwrap();
        assert_eq!(received_change, &expected_cache_change);
    }
}
