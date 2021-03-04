use crate::{
    behavior::{cache_change_from_data, Reader},
    messages::{submessages::Data, RtpsSubmessage},
    structure::HistoryCache,
    types::{constants::ENTITYID_UNKNOWN, GuidPrefix},
};

use super::StatelessReader;

pub struct BestEffortStatelessReaderBehavior;

impl BestEffortStatelessReaderBehavior {
    pub fn try_process_message<R: Reader>(
        reader: &mut impl StatelessReader<R>,
        source_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
    ) {
        Self::waiting_state(reader, source_guid_prefix, submessage);
    }

    fn waiting_state<R: Reader>(
        reader: &mut impl StatelessReader<R>,
        source_guid_prefix: GuidPrefix,
        submessage: &mut Option<RtpsSubmessage>,
    ) {
        if let Some(inner_submessage) = submessage {
            if let RtpsSubmessage::Data(data) = inner_submessage {
                if reader.guid().entity_id() == data.reader_id || data.reader_id == ENTITYID_UNKNOWN
                {
                    if let RtpsSubmessage::Data(data) = submessage.take().unwrap() {
                        Self::transition_t2(reader, source_guid_prefix, data)
                    }
                }
            }
        }
    }

    fn transition_t2<R: Reader>(
        reader: &mut impl StatelessReader<R>,
        guid_prefix: GuidPrefix,
        data: Data,
    ) {
        let cache_change = cache_change_from_data(data, &guid_prefix);
        reader.reader_cache().add_change(cache_change);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         behavior::types::{constants::DURATION_ZERO, Duration},
//         types::{
//             constants::{
//                 ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
//                 ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_UNKNOWN,
//             },
//             ChangeKind, ReliabilityKind, TopicKind, GUID,
//         },
//     };

//     use crate::behavior::change_kind_to_status_info;
//     use crate::messages::submessages::data_submessage::Payload;
//     use crate::messages::submessages::submessage_elements::ParameterList;
//     use crate::messages::types::Endianness;
//     use crate::messages::types::KeyHash;

//     #[test]
//     fn process_none_submessage() {
//         let guid = GUID::new([1; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
//         let topic_kind = TopicKind::WithKey;
//         let reliability_level = ReliabilityKind::BestEffort;
//         let expects_inline_qos = false;
//         let heartbeat_response_delay = Duration::from_millis(500);
//         let heartbeat_supression_duration = DURATION_ZERO;
//         let mut stateless_reader = StatelessReader::new(
//             guid,
//             vec![],
//             vec![],
//             topic_kind,
//             reliability_level,
//             expects_inline_qos,
//             heartbeat_response_delay,
//             heartbeat_supression_duration,
//         );

//         let source_guid_prefix = [5; 12];
//         let mut submessage = None;
//         BestEffortStatelessReaderBehavior::try_process_message(
//             &mut stateless_reader,
//             source_guid_prefix,
//             &mut submessage,
//         );
//     }

//     #[test]
//     fn process_data_submessage() {
//         let guid = GUID::new([1; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
//         let topic_kind = TopicKind::WithKey;
//         let reliability_level = ReliabilityKind::BestEffort;
//         let expects_inline_qos = false;
//         let heartbeat_response_delay = Duration::from_millis(500);
//         let heartbeat_supression_duration = DURATION_ZERO;
//         let mut stateless_reader = StatelessReader::new(
//             guid,
//             vec![],
//             vec![],
//             topic_kind,
//             reliability_level,
//             expects_inline_qos,
//             heartbeat_response_delay,
//             heartbeat_supression_duration,
//         );

//         let source_guid_prefix = [5; 12];
//         let status_info = change_kind_to_status_info(ChangeKind::Alive);
//         let key_hash = KeyHash([1; 16]);
//         let mut inline_qos = ParameterList::new();
//         inline_qos.parameter.push(key_hash.into());
//         inline_qos.parameter.push(status_info.into());
//         let data_submessage = Data::new(
//             Endianness::LittleEndian,
//             ENTITYID_UNKNOWN,
//             ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
//             1,
//             Some(inline_qos),
//             Payload::Data(vec![0, 1, 2]),
//         );
//         let expected_cache_change =
//             cache_change_from_data(data_submessage.clone(), &source_guid_prefix);

//         BestEffortStatelessReaderBehavior::try_process_message(
//             &mut stateless_reader,
//             source_guid_prefix,
//             &mut Some(RtpsSubmessage::Data(data_submessage)),
//         );
//         let received_change = stateless_reader.reader.reader_cache.get_change(1).unwrap();
//         assert_eq!(received_change, &expected_cache_change);
//     }
// }
