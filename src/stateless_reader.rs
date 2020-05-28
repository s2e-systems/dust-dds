use std::convert::TryFrom;
use crate::cache::{HistoryCache, CacheChange};
use crate::types::{Duration, LocatorList, ReliabilityKind, TopicKind, GUID, ChangeKind, StatusInfo, Parameter};
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::messages::{RtpsMessage, RtpsSubmessage};
use crate::inline_qos::InlineQosParameter;

pub struct StatelessReader {
    // Heartbeats are not relevant to stateless readers (only to readers),
    // hence the heartbeat_ members are not included here
    // heartbeat_response_delay: Duration,
    // heartbeat_suppression_duration: Duration,
    reader_cache: HistoryCache,
    expects_inline_qos: bool,
    // Enpoint members:
    /// Entity base class (contains the GUID)
    guid: GUID,
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,
    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    unicast_locator_list: LocatorList,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    multicast_locator_list: LocatorList,
}

impl StatelessReader {

    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
        expects_inline_qos: bool,
    ) -> Self {
        assert!(reliability_level == ReliabilityKind::BestEffort);
        StatelessReader {
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            reader_cache: HistoryCache::new(),
            expects_inline_qos,
        }
    }

    pub fn history_cache(&self) -> &HistoryCache {
        &self.reader_cache
    }

    pub fn process_message(&mut self, msg: &RtpsMessage) {

        let guid_prefix = *msg.header().guid_prefix();
        let mut _source_time = None;

        for submessage in msg.submessages().iter() {
            if let RtpsSubmessage::Data(data) = submessage {
                // Check if the message is for this reader and process it if that is the case
                if data.reader_id() == &ENTITYID_UNKNOWN {
                    let change_kind = if data.data_flag() && !data.key_flag() {
                        ChangeKind::Alive
                    } else if !data.data_flag() && data.key_flag() {
                        let status_info_qos_parameter = data.inline_qos().as_ref().unwrap().find_parameter(InlineQosParameter::StatusInfo(StatusInfo([0;4])).parameter_id()).unwrap();
                        if let InlineQosParameter::StatusInfo(status_info) = status_info_qos_parameter {
                            ChangeKind::try_from(*status_info).unwrap()
                        } else {
                            panic!("Status info not present");
                        }
                    }
                    else {
                        panic!("Combination should not occur");
                    };
                    
                    let cache_change = CacheChange::new(
                        change_kind,
                        GUID::new(guid_prefix, *data.writer_id() ),
                        [0;16],/*TODO: *data.key_hash() */
                        *data.writer_sn(),
                        None,
                        None,
                    );

                    self.reader_cache.add_change(cache_change);
                }
            }
            else if let RtpsSubmessage::InfoTs(infots) = submessage {
                _source_time = *infots.get_timestamp();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::types::constants::*;
    use crate::messages::{Data, Payload, Header};
    use crate::serdes::EndianessFlag;

    #[test]
    fn test_reader_process_data() {
        let data1 = Data::new(
            EndianessFlag::LittleEndian,
            ENTITYID_UNKNOWN, /*reader_id*/
            ENTITYID_UNKNOWN,/*writer_id*/
            SequenceNumber(1), /*writer_sn*/
            None, /*inline_qos*/
            Payload::Data(SerializedPayload(vec![0,1,2])),
        );

        let mut message = RtpsMessage::new(Header::new(GuidPrefix([2;12])), Vec::new());

        message.push(RtpsSubmessage::Data(data1));

        let mut reader = StatelessReader::new(
            GUID::new(GuidPrefix([0;12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0;16])],
            vec![],
            false,
           );

        assert_eq!(reader.history_cache().get_changes().len(), 0);

        reader.process_message(&message);

        assert_eq!(reader.history_cache().get_changes().len(), 1);
    }
}
