use rust_rtps::{StatelessWriter, StatelessReader, RtpsMessage, RtpsCompose, RtpsParse};
use rust_rtps::types::{ChangeKind, TopicKind, ReliabilityKind, Locator, GUID, GuidPrefix};
use rust_rtps::types::constants::{DURATION_ZERO, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER};

#[test]
fn test_stateless_writer_stateless_reader_direct_communication_integration() {
    let mut writer = StatelessWriter::new(
        GUID::new(GuidPrefix([0;12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
        TopicKind::WithKey,
        ReliabilityKind::BestEffort,
        vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
        vec![], /*multicast_locator_list*/
        false, /*push_mode*/
        DURATION_ZERO,  /* heartbeat_period */
        DURATION_ZERO, /* nack_response_delay */
        DURATION_ZERO, /* nack_suppression_duration */
       );

    let mut reader = StatelessReader::new(
        GUID::new(GuidPrefix([0;12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
        TopicKind::WithKey,
        ReliabilityKind::BestEffort,
        vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
        vec![], /*multicast_locator_list*/
        DURATION_ZERO, /*heartbeat_response_delay */
        DURATION_ZERO, /* heartbeat_response_delay */
        false,
       );

   let locator = Locator::new(0, 7400, [1;16]);

   writer.reader_locator_add(locator);

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

   writer.history_cache().add_change(cache_change_seq1.clone());
   writer.history_cache().add_change(cache_change_seq2.clone());

   let writer_data = writer.get_data_to_send(locator);

   reader.process_message(&writer_data);

   let reader_changes = reader.history_cache().get_changes();
   assert_eq!(reader_changes.len(), 2);
   assert!(reader_changes.contains(&cache_change_seq1));
   assert!(reader_changes.contains(&cache_change_seq2));
}

#[test]
fn test_stateless_writer_stateless_reader_serialized_communication_integration() {
    let mut writer = StatelessWriter::new(
        GUID::new(GuidPrefix([0;12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
        TopicKind::WithKey,
        ReliabilityKind::BestEffort,
        vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
        vec![], /*multicast_locator_list*/
        false, /*push_mode*/
        DURATION_ZERO,  /* heartbeat_period */
        DURATION_ZERO, /* nack_response_delay */
        DURATION_ZERO, /* nack_suppression_duration */
       );

    let mut reader = StatelessReader::new(
        GUID::new(GuidPrefix([0;12]), ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER),
        TopicKind::WithKey,
        ReliabilityKind::BestEffort,
        vec![Locator::new(0, 7400, [0;16])], /*unicast_locator_list*/
        vec![], /*multicast_locator_list*/
        DURATION_ZERO, /*heartbeat_response_delay */
        DURATION_ZERO, /* heartbeat_response_delay */
        false,
       );

   let locator = Locator::new(0, 7400, [1;16]);

   writer.reader_locator_add(locator);

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

   writer.history_cache().add_change(cache_change_seq1.clone());
   writer.history_cache().add_change(cache_change_seq2.clone());

   let writer_message = writer.get_data_to_send(locator);
   let mut buf  = Vec::new();
   writer_message.compose(&mut buf).unwrap();
  
   let received_message = RtpsMessage::parse(&buf).unwrap();
   reader.process_message(&received_message);

   let reader_changes = reader.history_cache().get_changes();
   assert_eq!(reader_changes.len(), 2);
   assert!(reader_changes.contains(&cache_change_seq1));
   assert!(reader_changes.contains(&cache_change_seq2));
}