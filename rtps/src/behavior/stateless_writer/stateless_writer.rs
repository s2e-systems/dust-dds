use std::sync::Mutex;
use std::collections::HashMap;

use crate::structure::{RtpsEndpoint, RtpsEntity};
use crate::behavior::DestinedMessages;
use crate::types::{Locator, ReliabilityKind, GUID, };
use super::reader_locator::ReaderLocator;

use rust_dds_interface::types::{ChangeKind, InstanceHandle, SequenceNumber, TopicKind, ParameterList};
use rust_dds_interface::history_cache::HistoryCache;
use rust_dds_interface::cache_change::CacheChange;

pub struct StatelessWriter {
    /// Entity base class (contains the GUID)
    guid: GUID,
    // entity: Entity,

    // Endpoint base class:
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,

    // The stateless writer does not receive messages so these fields are left out
    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    // unicast_locator_list: Vec<Locator>,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    // multicast_locator_list: Vec<Locator>,
    
    last_change_sequence_number: Mutex<SequenceNumber>,
    writer_cache: Mutex<HistoryCache>,
    data_max_sized_serialized: Option<i32>,

    reader_locators: Mutex<HashMap<Locator, ReaderLocator>>,
}

impl StatelessWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        writer_cache: HistoryCache,
    ) -> Self {
        assert!(reliability_level == ReliabilityKind::BestEffort, "Only BestEffort is supported on stateless writer");

        Self {
            guid,
            topic_kind,
            reliability_level,
            last_change_sequence_number: Mutex::new(0),
            writer_cache: Mutex::new(writer_cache),
            data_max_sized_serialized: None,
            reader_locators: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_change(
        &self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        let mut last_change_sequence_number = self.last_change_sequence_number.lock().unwrap();
        *last_change_sequence_number += 1;

        CacheChange::new(
            kind,
            self.guid.into(),
            handle,
            *last_change_sequence_number,
            data,
            inline_qos,
        )
    }

    pub fn produce_messages(&self) -> Vec<DestinedMessages> {
        let mut reader_locators = self.reader_locators.lock().unwrap();
        let mut output = Vec::new();
        for (&locator, reader_locator) in reader_locators.iter_mut() {
            let messages = reader_locator.produce_messages(&self.writer_cache.lock().unwrap(), *self.last_change_sequence_number.lock().unwrap());
            if !messages.is_empty() {
                output.push(DestinedMessages::SingleDestination{locator, messages});
            }
        }
        output
    } 

    pub fn writer_cache(&self) -> &Mutex<HistoryCache> {
        &self.writer_cache
    }

    pub fn reader_locator_add(&self, a_locator: Locator) {
        self.reader_locators.lock().unwrap().insert(a_locator, ReaderLocator::new(a_locator, self.guid.entity_id(), false /*expects_inline_qos*/));
    }

    pub fn reader_locator_remove(&self, a_locator: &Locator) {
        self.reader_locators.lock().unwrap().remove(a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        let mut reader_locators = self.reader_locators.lock().unwrap();
        for (_, rl) in reader_locators.iter_mut() {
            rl.unsent_changes_reset();
        }
    }
}

impl RtpsEntity for StatelessWriter {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl RtpsEndpoint for StatelessWriter {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::*;
    use crate::types::*;

    #[test]
    fn new_change() {
        let writer = StatelessWriter::new(
            GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            HistoryCache::default(),
        );

        let cache_change_seq1 = writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]), 
            None,                
            [1; 16],             
        );

        let cache_change_seq2 = writer.new_change(
            ChangeKind::NotAliveUnregistered,
            None,    
            None,    
            [1; 16], 
        );

        assert_eq!(cache_change_seq1.sequence_number(), 1);
        assert_eq!(cache_change_seq1.change_kind(), ChangeKind::Alive);
        assert_eq!(cache_change_seq1.inline_qos().unwrap().parameter.len(), 0);
        assert_eq!(cache_change_seq1.instance_handle(), [1; 16]);

        assert_eq!(cache_change_seq2.sequence_number(), 2);
        assert_eq!(
            cache_change_seq2.change_kind(),
            ChangeKind::NotAliveUnregistered
        );
        assert_eq!(cache_change_seq2.inline_qos().unwrap().parameter.len(), 0);
        assert_eq!(cache_change_seq2.instance_handle(), [1; 16]);
    }

    // #[test]
    // fn stateless_writer_run() {
    //     // Create the stateless writer
    //     let mut stateless_writer = StatelessWriter::new(
    //         GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
    //         TopicKind::WithKey,
    //         ReliabilityKind::BestEffort,
    //         HistoryCacheResourceLimits::default(),
    //     );

    //     // Add two locators
    //     let locator1 = Locator::new(0, 7400, [1; 16]);
    //     let locator2 = Locator::new(0, 7500, [2; 16]);
    //     stateless_writer.reader_locator_add(locator1);
    //     stateless_writer.reader_locator_add(locator2);

    //     let _cache_change_seq1 = stateless_writer.new_change(
    //         ChangeKind::Alive,
    //         Some(vec![1, 2, 3]), 
    //         None,                
    //         [1; 16],             
    //     );

    //     let cache_change_seq2 = stateless_writer.new_change(
    //         ChangeKind::Alive,
    //         Some(vec![4, 5, 6]), 
    //         None,                
    //         [1; 16],             
    //     );

    //     // stateless_writer.writer_cache().add_change(cache_change_seq1).unwrap();
    //     stateless_writer.writer_cache().add_change(cache_change_seq2).unwrap();

    //     stateless_writer.run();

    //     todo!()

    //     // let mut send_messages = stateless_writer.pop_send_messages();
    //     // assert_eq!(send_messages.len(), 2);

    //     // // Check that the two reader locators have messages sent to them. The order is not fixed so it can
    //     // // not be used for the test
    //     // send_messages.iter().find(|(dst_locator, _)| dst_locator == &vec![locator1]).unwrap();
    //     // send_messages.iter().find(|(dst_locator, _)| dst_locator == &vec![locator2]).unwrap();

    //     // let (_, send_messages_reader_locator_1) = send_messages.pop().unwrap();
    //     // let (_, send_messages_reader_locator_2) = send_messages.pop().unwrap();

    //     // // Check that the same messages are sent to both locators
    //     // assert_eq!(send_messages_reader_locator_1, send_messages_reader_locator_2);

    //     // if let RtpsSubmessage::Gap(_) = &send_messages_reader_locator_1[0] {
    //     //     // The contents of the message are tested in the reader locator so simply assert the type is correct
    //     //     assert!(true)
    //     // } else {
    //     //     panic!("Wrong message type");
    //     // };

    //     // if let RtpsSubmessage::Data(_) = &send_messages_reader_locator_1[1] {
    //     //         // The contents of the message are tested in the reader locator so simply assert the type is correct
    //     //         assert!(true)
    //     // } else {
    //     //     panic!("Wrong message type");
    //     // };

    //     // // Test that nothing more is sent after the first time
    //     // stateless_writer.run();
    //     // assert_eq!(stateless_writer.pop_send_messages().len(), 0);
    // }
}
