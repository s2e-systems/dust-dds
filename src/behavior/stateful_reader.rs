use crate::types::{GUID, SequenceNumber};
use crate::messages::{RtpsMessage, RtpsSubmessage};
use crate::cache::{HistoryCache};
use crate::stateful_reader::WriterProxy;
use crate::inline_qos_types::{KeyHash, StatusInfo, };
use super::cache_change_from_data;

pub struct StatefulReaderBehaviour {}

impl StatefulReaderBehaviour {
    pub fn run_best_effort(writer_proxy: &mut WriterProxy, _reader_guid: &GUID, history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>) {
        StatefulReaderBehaviour::run_waiting_state(writer_proxy, history_cache, received_message);
    }

    pub fn run_waiting_state(writer_proxy: &mut WriterProxy, history_cache: &mut HistoryCache, received_message: Option<&RtpsMessage>) {
        if let Some(received_message) = received_message {
            let guid_prefix = received_message.header().guid_prefix();
            for submessage in received_message.submessages().iter() {                
                if let RtpsSubmessage::Data(data) = submessage {
                    let expected_seq_number = writer_proxy.available_changes_max() + 1;
                    if data.writer_sn() >= &expected_seq_number {
                        let cache_change = cache_change_from_data(data, guid_prefix);
                        history_cache.add_change(cache_change);
                        writer_proxy.received_change_set(*data.writer_sn());
                        writer_proxy.lost_changes_update(*data.writer_sn());
                    }
                } else if let RtpsSubmessage::Gap(_gap) = submessage {
                    let _expected_seq_number = writer_proxy.available_changes_max() + 1;
                    todo!()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SequenceNumber, ChangeKind, GuidPrefix, TopicKind, ReliabilityKind, Locator};
    use crate::behavior::types::constants::DURATION_ZERO;
    use crate::messages::types::Count;
    use crate::types::constants::{
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, 
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, };
    use crate::cache::CacheChange;
    use crate::messages::{Data, Payload};
    use crate::messages::submessage_elements::{SequenceNumberSet, Parameter, ParameterList};
    use crate::serdes::Endianness;
    use crate::stateful_writer::StatefulWriter;
    use crate::serialized_payload::SerializedPayload;
    use std::thread::sleep;

    #[test]
    fn run_waiting_state() {
        let mut history_cache = HistoryCache::new();
        let remote_writer_guid = GUID::new(GuidPrefix([1;12]), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        let mut submessages = Vec::new();
        let inline_qos_parameters = vec![
            Parameter::new(StatusInfo::from(ChangeKind::Alive), Endianness::LittleEndian),
            Parameter::new(KeyHash([1;16]), Endianness::LittleEndian)];

        let data1 = Data::new(
            Endianness::LittleEndian, 
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, 
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER, 
            SequenceNumber(1),
            Some(ParameterList::new(inline_qos_parameters)),
            Payload::Data(SerializedPayload(vec![1,2,3])));
        submessages.push(RtpsSubmessage::Data(data1));

        let received_message = RtpsMessage::new(*remote_writer_guid.prefix(), submessages);

        StatefulReaderBehaviour::run_waiting_state(&mut writer_proxy, &mut history_cache, Some(&received_message));
    }
}