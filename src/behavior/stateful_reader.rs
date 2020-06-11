use crate::types::{GUID, SequenceNumber};
use crate::messages::{RtpsMessage, RtpsSubmessage};
use crate::cache::{HistoryCache};
use crate::stateful_reader::WriterProxy;
use super::cache_change_from_data;

pub struct StatefulReaderBehaviour {}

impl StatefulReaderBehaviour {
    pub fn run_best_effort(_writer_proxy: &mut WriterProxy, _reader_guid: &GUID, _history_cache: &HistoryCache, _last_change_sequence_number: SequenceNumber) -> Option<Vec<RtpsSubmessage>> {
        todo!()
    }

    pub fn run_waiting_state(writer_proxy: &mut WriterProxy, history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>) {
        if let Some(received_message) = received_message {
            let guid_prefix = received_message.header().guid_prefix();
            for submessage in received_message.submessages().iter() {                
                if let RtpsSubmessage::Data(data) = submessage {
                    let expected_seq_number = *writer_proxy.available_changes_max() + 1;
                    if data.writer_sn() >= &expected_seq_number {
                        let cache_change = cache_change_from_data(data, guid_prefix);
                        history_cache.add_change(cache_change);
                        writer_proxy.received_change_set(*data.writer_sn());
                        writer_proxy.lost_changes_update(*data.writer_sn());
                    }
                } else if let RtpsSubmessage::Gap(_gap) = submessage {
                    let _expected_seq_number = *writer_proxy.available_changes_max() + 1;
                    todo!()
                }
            }
        }

        todo!()
    }
}