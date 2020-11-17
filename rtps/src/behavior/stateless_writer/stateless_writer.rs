use std::sync::Mutex;
use std::collections::HashMap;

use crate::behavior::{RtpsWriter, DestinedMessages};
use crate::types::{Locator, ReliabilityKind, };
use super::reader_locator::ReaderLocator;

use rust_dds_interface::history_cache::HistoryCache;

pub struct StatelessWriter {
    pub writer: RtpsWriter,
    reader_locators: Mutex<HashMap<Locator, ReaderLocator>>,
}

impl StatelessWriter {
    pub fn new(
        writer: RtpsWriter,
    ) -> Self {
        assert!(writer.endpoint.reliability_level == ReliabilityKind::BestEffort, "Only BestEffort is supported on stateless writer");

        Self {
            writer,
            reader_locators: Mutex::new(HashMap::new()),
        }
    }

    pub fn produce_messages(&self, writer_cache: &Mutex<HistoryCache>) -> Vec<DestinedMessages> {
        let mut reader_locators = self.reader_locators.lock().unwrap();
        let mut output = Vec::new();
        for (&locator, reader_locator) in reader_locators.iter_mut() {
            let messages = reader_locator.produce_messages(&writer_cache.lock().unwrap(), self.writer.last_change_sequence_number);
            if !messages.is_empty() {
                output.push(DestinedMessages::SingleDestination{locator, messages});
            }
        }
        output
    } 

    pub fn reader_locator_add(&self, a_locator: Locator) {
        self.reader_locators.lock().unwrap().insert(a_locator, ReaderLocator::new(a_locator, self.writer.endpoint.entity.guid.entity_id(), false /*expects_inline_qos*/));
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::types::constants::*;
//     use crate::types::*;

//     #[test]
//     fn new_change() {
//         let writer = StatelessWriter::new(
//             GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
//             TopicKind::WithKey,
//             ReliabilityKind::BestEffort,
//             HistoryCache::default(),
//         );

//         let cache_change_seq1 = writer.new_change(
//             ChangeKind::Alive,
//             Some(vec![1, 2, 3]), 
//             None,                
//             [1; 16],             
//         );

//         let cache_change_seq2 = writer.new_change(
//             ChangeKind::NotAliveUnregistered,
//             None,    
//             None,    
//             [1; 16], 
//         );

//         assert_eq!(cache_change_seq1.sequence_number(), 1);
//         assert_eq!(cache_change_seq1.change_kind(), ChangeKind::Alive);
//         assert_eq!(cache_change_seq1.inline_qos().unwrap().parameter.len(), 0);
//         assert_eq!(cache_change_seq1.instance_handle(), [1; 16]);

//         assert_eq!(cache_change_seq2.sequence_number(), 2);
//         assert_eq!(
//             cache_change_seq2.change_kind(),
//             ChangeKind::NotAliveUnregistered
//         );
//         assert_eq!(cache_change_seq2.inline_qos().unwrap().parameter.len(), 0);
//         assert_eq!(cache_change_seq2.instance_handle(), [1; 16]);
//     }

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
// }
