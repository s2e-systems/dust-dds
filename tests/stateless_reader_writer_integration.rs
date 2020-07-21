use rust_rtps::{StatelessWriter, StatelessReader, };
use rust_rtps::types::{ChangeKind, TopicKind, Locator, GUID, };
use rust_rtps::types::constants::{ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, };
use rust_rtps::{ParameterId, Pid};

use serde::{Serialize, };

#[derive(Debug, Serialize)]
struct SpecialQos(u16);

impl Pid for SpecialQos{
    fn pid() -> ParameterId {
        0x0AA0
    }
}

#[derive(Debug, Serialize)]
struct OtherQos(i32);

impl Pid for OtherQos{
    fn pid() -> ParameterId {
        0x0AA1
    }
}

#[test]
fn test_stateless_writer_stateless_reader_direct_communication_integration() {
    let writer_guid = GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
    let reader_guid = GUID::new([0;12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
    let destination_locator = Locator::new(5, 7400, [1;16]);

    let mut writer = StatelessWriter::new(
        writer_guid,
        TopicKind::WithKey,
       );

    let mut reader = StatelessReader::new(
        reader_guid,
        TopicKind::WithKey,
        vec![destination_locator],
        vec![],
        false,
       );

   writer.reader_locator_add(destination_locator);

   let cache_change_seq1 = writer.new_change(
       ChangeKind::Alive,
       Some(vec![1,2,3]), /*data*/
       None, /*inline_qos*/
       [0;16], /*handle*/
   );
   
   let cache_change_seq2 = writer.new_change(
       ChangeKind::Alive,
       Some(vec!(4,5,6)), /*data*/
       None, /*inline_qos*/
       [0;16], /*handle*/
   );

   let cache_change_seq3 = writer.new_change(
    ChangeKind::NotAliveUnregistered,
    None, /*data*/
    None, /*inline_qos*/
    [0;16], /*handle*/
    );

    let cache_change_seq4 = writer.new_change(
        ChangeKind::NotAliveDisposed,
        None, /*data*/
        None, /*inline_qos*/
        [0;16], /*handle*/
    );

   writer.history_cache().add_change(cache_change_seq1.clone());
   writer.history_cache().add_change(cache_change_seq2.clone());
   writer.history_cache().add_change(cache_change_seq3.clone());
   writer.history_cache().add_change(cache_change_seq4.clone());

   writer.run();

   reader.push_receive_message(*writer_guid.prefix(), writer.reader_locators().get(&destination_locator).unwrap().pop_send_message().unwrap());
   reader.push_receive_message(*writer_guid.prefix(), writer.reader_locators().get(&destination_locator).unwrap().pop_send_message().unwrap());
   reader.push_receive_message(*writer_guid.prefix(), writer.reader_locators().get(&destination_locator).unwrap().pop_send_message().unwrap());
   reader.push_receive_message(*writer_guid.prefix(), writer.reader_locators().get(&destination_locator).unwrap().pop_send_message().unwrap());

   reader.run();

   let reader_changes = reader.history_cache().changes();
   assert_eq!(reader_changes.len(), 4);
   assert!(reader_changes.contains(&cache_change_seq1));
   assert!(reader_changes.contains(&cache_change_seq2));
   assert!(reader_changes.contains(&cache_change_seq3));
   assert!(reader_changes.contains(&cache_change_seq4));
}