use crate::cache::HistoryCache;
use crate::messages::{RtpsMessage, RtpsSubmessage};
use crate::types::constants::ENTITYID_UNKNOWN;

use super::cache_change_from_data;

pub struct StatelessReaderBehavior {}

impl StatelessReaderBehavior {
    pub fn run_best_effort(history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>){
        StatelessReaderBehavior::run_waiting_state(history_cache, received_message);
    }

    pub fn run_waiting_state(history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>) {

        if let Some(received_message) = received_message {
            let guid_prefix = received_message.header().guid_prefix();
            let mut _source_time = None;

            for submessage in received_message.submessages().iter() {
                if let RtpsSubmessage::Data(data) = submessage {
                    // Check if the message is for this reader and process it if that is the case
                    if data.reader_id() == &ENTITYID_UNKNOWN {

                        let cache_change = cache_change_from_data(data, guid_prefix);
                        history_cache.add_change(cache_change);
                    }
                }
                else if let RtpsSubmessage::InfoTs(infots) = submessage {
                    _source_time = *infots.get_timestamp();
                }
            }
        }
    }
}