use crate::{
    behavior::cache_change_from_data,
    messages::{
        submessages::{Data, Gap},
        RtpsSubmessage,
    },
    structure::HistoryCache,
    types::{GuidPrefix, GUID},
};

use super::WriterProxy;

pub struct BestEffortWriterProxyBehavior;

impl BestEffortWriterProxyBehavior {

    pub fn try_process_message(
        writer_proxy: &mut WriterProxy,
        src_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
        history_cache: &mut HistoryCache,
    ) {
        Self::waiting_state(writer_proxy, src_guid_prefix, submessage, history_cache);
    }

    fn waiting_state(
        writer_proxy: &mut WriterProxy,
        src_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
        history_cache: &mut HistoryCache,
    ) {
        if let Some(inner_submessage) = submessage {
            if Self::is_submessage_destination(writer_proxy, src_guid_prefix, inner_submessage) {
                match submessage.take().unwrap() {
                    RtpsSubmessage::Data(data) => Self::transition_t2(writer_proxy, history_cache, data),
                    RtpsSubmessage::Gap(gap) => Self::transition_t4(writer_proxy, gap),
                    _ => panic!("Unexpected reader message received"),
                }
            }
        }
    }

    fn transition_t2(writer_proxy: &mut WriterProxy, history_cache: &mut HistoryCache, data: Data) {
        let expected_seq_number = writer_proxy.available_changes_max() + 1;
        if data.writer_sn() >= expected_seq_number {
            writer_proxy.received_change_set(data.writer_sn());
            writer_proxy.lost_changes_update(data.writer_sn());
            let cache_change = cache_change_from_data(data, &writer_proxy.remote_writer_guid.prefix());
            history_cache.add_change(cache_change);
        }
    }

    fn transition_t4(writer_proxy: &mut WriterProxy, gap: Gap) {
        for seq_num in gap.gap_start()..gap.gap_list().base() - 1 {
            writer_proxy.irrelevant_change_set(seq_num);
        }

        for &seq_num in gap.gap_list().set() {
            writer_proxy.irrelevant_change_set(seq_num);
        }
    }

    fn is_submessage_destination(
        writer_proxy: &mut WriterProxy,
        src_guid_prefix: GuidPrefix,
        submessage: &RtpsSubmessage,
    ) -> bool {
        let writer_id = match submessage {
            RtpsSubmessage::Data(data) => data.writer_id(),
            RtpsSubmessage::Gap(gap) => gap.writer_id(),
            _ => return false,
        };

        let writer_guid = GUID::new(src_guid_prefix, writer_id);
        if writer_proxy.remote_writer_guid == writer_guid {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{behavior::change_kind_to_status_info, messages::submessages::submessage_elements::ParameterList, types::ChangeKind};
    use crate::messages::submessages::data_submessage::Payload;
    use crate::messages::types::{Endianness, KeyHash};
    use crate::types::constants::{
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    };
    use crate::types::Locator;

    #[test]
    fn process_none_submessage() {
        let remote_writer_guid = GUID::new([5; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let unicast_locator_list = vec![Locator::new_udpv4(7400, [127, 0, 0, 1])];
        let multicast_locator_list = vec![];
        let mut writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );

        let mut history_cache = HistoryCache::default();

        let source_guid_prefix = [5; 12];
        let mut submessage = None;
        BestEffortWriterProxyBehavior::try_process_message(
            &mut writer_proxy,
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
        let mut writer_proxy = WriterProxy::new(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
        );

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

        BestEffortWriterProxyBehavior::try_process_message(
            &mut writer_proxy,
            source_guid_prefix,
            &mut Some(RtpsSubmessage::Data(data_submessage)),
            &mut history_cache,
        );
        let received_change = history_cache.get_change(1).unwrap();
        assert_eq!(received_change, &expected_cache_change);
    }
}
