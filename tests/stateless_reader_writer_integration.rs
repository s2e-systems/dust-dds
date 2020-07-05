use rust_rtps::{StatelessWriter, StatelessReader, RtpsMessage, RtpsCompose, RtpsParse, RtpsSerialize};
use rust_rtps::types::{ChangeKind, TopicKind, ReliabilityKind, Locator, GUID, GuidPrefix, };
use rust_rtps::types::constants::{ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, };
use rust_rtps::behavior::types::constants::DURATION_ZERO;
use rust_rtps::{ParameterId, Endianness, RtpsSerdesResult, ParameterList, Pid};

#[derive(Debug)]
struct SpecialQos(u16);

impl Pid for SpecialQos{
    fn pid() -> ParameterId where Self: Sized {
        0x0AA0
    }
}

impl RtpsSerialize for SpecialQos {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        match endianness {
            Endianness::BigEndian => writer.write(&self.0.to_be_bytes())?,
            Endianness::LittleEndian => writer.write(&self.0.to_le_bytes())?,
        };

        Ok(())
    }
}

#[derive(Debug)]
struct OtherQos(i32);

impl Pid for OtherQos{
    fn pid() -> ParameterId where Self: Sized {
        0x0AA1
    }
}

impl RtpsSerialize for OtherQos {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        match endianness {
            Endianness::BigEndian => writer.write(&self.0.to_be_bytes())?,
            Endianness::LittleEndian => writer.write(&self.0.to_le_bytes())?,
        };

        Ok(())
    }
}

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
        vec![Locator::new(0, 7400, [0;16])],
        vec![],
        false,
       );

   let locator = Locator::new(0, 7400, [1;16]);

   writer.reader_locator_add(locator);

   let mut inline_qos = ParameterList::new();
   inline_qos.push(SpecialQos(10));

   let cache_change_seq1 = writer.new_change(
       ChangeKind::Alive,
       Some(vec![1,2,3]), /*data*/
       Some(inline_qos), /*inline_qos*/
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

   let writer_data = writer.run(&locator).unwrap();

   reader.run(Some(&writer_data));

   let reader_changes = reader.history_cache().get_changes();
   assert_eq!(reader_changes.len(), 4);
   assert!(reader_changes.contains(&cache_change_seq1));
   assert!(reader_changes.contains(&cache_change_seq2));
   assert!(reader_changes.contains(&cache_change_seq3));
   assert!(reader_changes.contains(&cache_change_seq4));
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
        vec![Locator::new(0, 7400, [0;16])],
        vec![],
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

   let writer_message = writer.run(&locator).unwrap();
   let mut buf  = Vec::new();
   writer_message.compose(&mut buf).unwrap();
  
   let received_message = RtpsMessage::parse(&buf).unwrap();
   reader.run(Some(&received_message));

   let reader_changes = reader.history_cache().get_changes();
   assert_eq!(reader_changes.len(), 4);
   assert!(reader_changes.contains(&cache_change_seq1));
   assert!(reader_changes.contains(&cache_change_seq2));
   assert!(reader_changes.contains(&cache_change_seq3));
   assert!(reader_changes.contains(&cache_change_seq4));
}