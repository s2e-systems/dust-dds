use rust_rtps::{StatelessWriter, StatelessReader, };
use rust_rtps::types::{ChangeKind, TopicKind, Locator, GUID, };
use rust_rtps::types::constants::{ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, };
use rust_rtps::{ParameterId, Pid, MemoryTransport, RtpsMessageSender, RtpsMessageReceiver};

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
    let guid_prefix = [0;12];
    let writer_guid = GUID::new(guid_prefix, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
    let reader_guid = GUID::new(guid_prefix, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER);
    let source_locator = Locator::new(5, 7400, [2;16]);
    let destination_locator = Locator::new(5, 7400, [1;16]);

    let memory_transport1 = MemoryTransport::new(source_locator, vec![]).unwrap();
    let memory_transport2 = MemoryTransport::new(destination_locator, vec![]).unwrap();

    let writer = StatelessWriter::new(
        writer_guid,
        TopicKind::WithKey,
       );

    let reader = StatelessReader::new(
        reader_guid,
        TopicKind::WithKey,
        vec![source_locator],
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

   writer.writer_cache().add_change(cache_change_seq1);
   writer.writer_cache().add_change(cache_change_seq2);
   writer.writer_cache().add_change(cache_change_seq3);
   writer.writer_cache().add_change(cache_change_seq4);

   writer.run();

   RtpsMessageSender::send(guid_prefix, &memory_transport1, &[&writer]);

   memory_transport2.receive_from(&memory_transport1);

   RtpsMessageReceiver::receive(guid_prefix, &memory_transport2, &[&reader]);

   reader.run();

   let reader_changes = reader.reader_cache().changes();
   assert_eq!(reader_changes.len(), 4);
   assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 1).is_some());
   assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 2).is_some());
   assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 3).is_some());
   assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 4).is_some());
}