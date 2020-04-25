use std::collections::HashMap;

use crate::cache::{CacheChange, HistoryCache};
use crate::endpoint::Endpoint;
use crate::entity::Entity;
use crate::messages::{RtpsSubmessage, InlineQosParameter, Payload};
use crate::proxy::WriterProxy;
use crate::types::{
    ChangeKind, Duration, EntityId, InlineQosParameterList, LocatorList, ParameterList,
    ReliabilityKind, SequenceNumber, TopicKind, GUID,
};

pub struct StatelessReader {
    heartbeat_response_delay: Duration,
    heartbeat_suppression_duration: Duration,
    reader_cache: HistoryCache,
    expects_inline_qos: bool,
    // Enpoint members:
    /// Entity base class (contains the GUID)
    entity: Entity,
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
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        StatelessReader {
            entity : Entity{ guid: guid},
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_response_delay,
            heartbeat_suppression_duration,
            reader_cache: HistoryCache::new(),
            expects_inline_qos,
        }
    }

    pub fn history_cache(&self) -> &HistoryCache {
        &self.reader_cache
    }

    pub fn process_data(&mut self, msg: &Vec<RtpsSubmessage>) {
        for item in msg.iter() {
            if let RtpsSubmessage::Data(data) = item {
                println!("Data message found");
            }
        }
    }

    pub fn read_data(
        &mut self,
        writer_guid: GUID,
        sequence_number: SequenceNumber,
        inline_qos: Option<InlineQosParameterList>,
        serialized_payload: Payload,
    ) {
        println!("Reader is processing data");

        if let Payload::Data(data) = serialized_payload {
            if let Some(inline_qos_list) = inline_qos {
                let key_hash_parameter = inline_qos_list.iter().find(|&x| x.is_key_hash());
                if let Some(InlineQosParameter::KeyHash(instance_handle)) = key_hash_parameter {
                    let rcc = CacheChange::new(
                        ChangeKind::Alive,
                        writer_guid,
                        *instance_handle,
                        sequence_number,
                        None, /*inline_qos*/
                        Some(data),
                    );
                    self.reader_cache.add_change(rcc);
                }
            }
        } else if let Payload::Key(key) = serialized_payload {
            if let Some(inline_qos_list) = inline_qos {
                let status_info_parameter = inline_qos_list.iter().find(|&x| x.is_status_info());
                if let Some(InlineQosParameter::StatusInfo(status_info)) = status_info_parameter {
                    // TODO: Check the liveliness changes to the entity
                }
            }
        } else {
            // TODO: Either no payload or non standardized payload. In either case, not implemented yet
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::messages::{Data, Payload};

    #[test]
    fn test_reader_process_data() {
        let data1 = Data::new(
            ENTITYID_UNKNOWN, /*reader_id*/
            ENTITYID_UNKNOWN,/*writer_id*/
            1, /*writer_sn*/
            None, /*inline_qos*/
            Payload::Data(vec![0,1,2]),
        );

        let mut message = Vec::new();
        message.push(RtpsSubmessage::Data(data1));

        let mut reader = StatelessReader::new(
            GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
            vec![], /*multicast_locator_list*/
            DURATION_ZERO, /*heartbeat_response_delay */
            DURATION_ZERO, /* heartbeat_response_delay */
            false,
           );

        reader.process_data(&message);
    }
}
