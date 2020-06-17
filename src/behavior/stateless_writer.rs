use crate::types::{GUID, SequenceNumber, ChangeKind};
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::{RtpsSubmessage, InfoTs, Data, Payload, Gap};
use crate::cache::{HistoryCache};
use crate::stateless_writer::ReaderLocator;
use crate::serdes::Endianness;
use crate::messages::types::{Time};
use crate::inline_qos_types::{KeyHash, StatusInfo};
use crate::messages::submessage_elements::{Parameter, ParameterList};
use crate::serialized_payload::SerializedPayload;

pub struct StatelessWriterBehavior {}

impl StatelessWriterBehavior{
    pub fn run_best_effort(reader_locator: &mut ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Option<Vec<RtpsSubmessage>> {
        if !reader_locator.unsent_changes(last_change_sequence_number).is_empty() {
            Some(StatelessWriterBehavior::run_pushing_state(reader_locator, writer_guid, history_cache, last_change_sequence_number))
        } else {
            None
        }
    }

    fn run_pushing_state(reader_locator: &mut ReaderLocator, writer_guid: &GUID, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) -> Vec<RtpsSubmessage> {

        // This state is only valid if there are unsent changes
        assert!(!reader_locator.unsent_changes(last_change_sequence_number).is_empty());
    
        let endianness = Endianness::LittleEndian;
        let mut submessages = Vec::with_capacity(2); // TODO: Probably can be preallocated with the correct size
    
        let time = Time::now();
        let infots = InfoTs::new(Some(time), endianness);
        submessages.push(RtpsSubmessage::InfoTs(infots));
    
        while let Some(next_unsent_seq_num) = reader_locator.next_unsent_change(last_change_sequence_number) {
            if let Some(cache_change) = history_cache
                .get_change_with_sequence_number(&next_unsent_seq_num)
            {
                let change_kind = *cache_change.change_kind();
    
                let mut parameter = Vec::new();
    
                let payload = match change_kind {
                    ChangeKind::Alive => {
                        parameter.push(Parameter::new(StatusInfo::from(change_kind), endianness));
                        parameter.push(Parameter::new(KeyHash(*cache_change.instance_handle()), endianness));
                        Payload::Data(SerializedPayload(cache_change.data_value().unwrap().to_vec()))
                    },
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
                        parameter.push(Parameter::new(StatusInfo::from(change_kind), endianness));
                        Payload::Key(SerializedPayload(cache_change.instance_handle().to_vec()))
                    }
                };
                let inline_qos_parameter_list = ParameterList::new(parameter);
                let data = Data::new(
                    Endianness::LittleEndian.into(),
                    ENTITYID_UNKNOWN,
                    *writer_guid.entity_id(),
                    *cache_change.sequence_number(),
                    Some(inline_qos_parameter_list), 
                    payload,
                );
    
                submessages.push(RtpsSubmessage::Data(data));
            } else {
                let gap = Gap::new(
                    ENTITYID_UNKNOWN, 
                    *writer_guid.entity_id(),
                    next_unsent_seq_num,
                    Endianness::LittleEndian);
    
                submessages.push(RtpsSubmessage::Gap(gap));
            }
        }
    
        submessages
    }
}
