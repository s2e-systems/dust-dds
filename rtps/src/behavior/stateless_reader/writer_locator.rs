use std::collections::VecDeque;

use crate::types::{GuidPrefix, GUID, };
use crate::structure::HistoryCache;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::{Data, };
use crate::behavior::cache_change_from_data;

pub struct WriterLocator {
    remote_writer_guid: GUID,
    received_messages: VecDeque<(GuidPrefix, RtpsSubmessage)>,
}

impl WriterLocator {
    pub fn run(&mut self, history_cache: &HistoryCache) {
        self.waiting_state(history_cache)
    }

    fn push_receive_message(&mut self, src_guid_prefix: GuidPrefix, submessage: RtpsSubmessage) {
        assert!(self.is_submessage_destination(&src_guid_prefix, &submessage));

        self.received_messages.push_back((src_guid_prefix, submessage));
    }

    fn is_submessage_destination(&self, src_guid_prefix: &GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        let writer_id = match submessage {
            RtpsSubmessage::Data(data) => data.writer_id(),
            _ => return false,
        };

        let writer_guid = GUID::new(*src_guid_prefix, writer_id);

        self.remote_writer_guid == writer_guid
    }

    fn waiting_state(&mut self, history_cache: &HistoryCache) {
        while let Some((guid_prefix, received_message)) =  self.received_messages.pop_front() {
            match received_message {
                RtpsSubmessage::Data(data) => Self::transition_t2(history_cache, guid_prefix, data),
                _ => (),
            };
        }
    }

    fn transition_t2(history_cache: &HistoryCache, guid_prefix: GuidPrefix, data: Data) {
        let cache_change = cache_change_from_data(data, &guid_prefix);
        history_cache.add_change(cache_change).unwrap();
    }
}