use crate::types::{GuidPrefix};
use crate::cache::HistoryCache;
use crate::messages::{Data};
use crate::messages::receiver::ReaderReceiveMessage;
use crate::stateless_reader::StatelessReader;

use super::cache_change_from_data;

pub struct BestEffortStatelessReaderBehavior {}

impl BestEffortStatelessReaderBehavior {
    pub fn run(reader: &mut StatelessReader){
        Self::waiting_state(reader);
    }

    fn waiting_state(reader: &mut StatelessReader) {
        while let Some((source_guid_prefix, received_message)) =  reader.pop_receive_message() {
            match received_message {
                ReaderReceiveMessage::Data(data) => Self::transition_t2(reader.mut_history_cache(), &data, &source_guid_prefix),
                _ => (),
            };
        }
    }

    fn transition_t2(history_cache: &mut HistoryCache, data: &Data, guid_prefix: &GuidPrefix) {
        let cache_change = cache_change_from_data(data, guid_prefix);
        history_cache.add_change(cache_change);
    }
}