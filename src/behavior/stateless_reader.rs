use crate::types::GuidPrefix;
use crate::messages::{Data, RtpsSubmessage};
use crate::messages::message_receiver::Receiver;
use crate::structure::stateless_reader::StatelessReader;

use super::cache_change_from_data;

pub struct BestEffortStatelessReaderBehavior {}

impl BestEffortStatelessReaderBehavior {
    pub fn run(reader: &StatelessReader){
        Self::waiting_state(reader);
    }

    fn waiting_state(reader: &StatelessReader) {
        while let Some((source_guid_prefix, received_message)) =  reader.pop_receive_message(reader.guid()) {
            match received_message {
                RtpsSubmessage::Data(data) => Self::transition_t2(reader, source_guid_prefix, data),
                _ => (),
            };
        }
    }

    fn transition_t2(reader: &StatelessReader, guid_prefix: GuidPrefix, data: Data) {
        let cache_change = cache_change_from_data(data, &guid_prefix);
        reader.reader_cache().add_change(cache_change);
    }
}