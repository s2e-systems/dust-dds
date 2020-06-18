use rust_rtps::behavior::types::constants::DURATION_ZERO;
use rust_rtps::types::constants::{
    ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
};
use rust_rtps::types::{ChangeKind, GuidPrefix, Locator, ReliabilityKind, TopicKind, GUID};
use rust_rtps::{
    ReaderProxy, RtpsCompose, RtpsMessage, RtpsParse, StatefulReader, StatefulWriter, WriterProxy,
};

#[test]
fn stateful_writer_stateful_reader_direct_communication_integration() {
    let writer_guid = GUID::new(
        GuidPrefix([0; 12]),
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    );
    let mut writer = StatefulWriter::new(
        writer_guid,
        TopicKind::WithKey,
        ReliabilityKind::BestEffort,
        vec![Locator::new(0, 7400, [0; 16])], /*unicast_locator_list*/
        vec![],                               /*multicast_locator_list*/
        false,                                /*push_mode*/
        DURATION_ZERO,                        /* heartbeat_period */
        DURATION_ZERO,                        /* nack_response_delay */
        DURATION_ZERO,                        /* nack_suppression_duration */
    );
    let reader_guid = GUID::new(
        GuidPrefix([0; 12]),
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    );
    let mut reader = StatefulReader::new(
        reader_guid,
        TopicKind::WithKey,
        ReliabilityKind::BestEffort,
        vec![Locator::new(0, 7400, [0; 16])],
        vec![],
        false,
        DURATION_ZERO,
    );

    let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);
    let writer_proxy = WriterProxy::new(writer_guid, vec![], vec![]);

    writer.matched_reader_add(reader_proxy);
    reader.matched_writer_add(writer_proxy);

    let cache_change_seq1 = writer.new_change(
        ChangeKind::Alive,
        Some(vec![1, 2, 3]), /*data*/
        None,                /*inline_qos*/
        [0; 16],             /*handle*/
    );

    let cache_change_seq2 = writer.new_change(
        ChangeKind::Alive,
        Some(vec![4, 5, 6]), /*data*/
        None,                /*inline_qos*/
        [0; 16],             /*handle*/
    );

    let cache_change_seq3 = writer.new_change(
        ChangeKind::NotAliveUnregistered,
        None,    /*data*/
        None,    /*inline_qos*/
        [0; 16], /*handle*/
    );

    let cache_change_seq4 = writer.new_change(
        ChangeKind::NotAliveDisposed,
        None,    /*data*/
        None,    /*inline_qos*/
        [0; 16], /*handle*/
    );

    writer.writer_cache().add_change(cache_change_seq1.clone());
    writer.writer_cache().add_change(cache_change_seq2.clone());
    writer.writer_cache().add_change(cache_change_seq3.clone());
    writer.writer_cache().add_change(cache_change_seq4.clone());

    let writer_data = writer.run(&reader_guid, None).unwrap();

    reader.run(&writer_guid, Some(&writer_data));

    let reader_changes = reader.reader_cache().get_changes();
    assert_eq!(reader_changes.len(), 4);
    assert!(reader_changes.contains(&cache_change_seq1));
    assert!(reader_changes.contains(&cache_change_seq2));
    assert!(reader_changes.contains(&cache_change_seq3));
    assert!(reader_changes.contains(&cache_change_seq4));
}

#[test]
fn stateful_writer_stateful_reader_serialized_communication_integration() {
    let writer_guid = GUID::new(
        GuidPrefix([0; 12]),
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    );
    let mut writer = StatefulWriter::new(
        writer_guid,
        TopicKind::WithKey,
        ReliabilityKind::BestEffort,
        vec![Locator::new(0, 7400, [0; 16])], /*unicast_locator_list*/
        vec![],                               /*multicast_locator_list*/
        false,                                /*push_mode*/
        DURATION_ZERO,                        /* heartbeat_period */
        DURATION_ZERO,                        /* nack_response_delay */
        DURATION_ZERO,                        /* nack_suppression_duration */
    );
    let reader_guid = GUID::new(
        GuidPrefix([0; 12]),
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    );
    let mut reader = StatefulReader::new(
        reader_guid,
        TopicKind::WithKey,
        ReliabilityKind::BestEffort,
        vec![Locator::new(0, 7400, [0; 16])],
        vec![],
        false,
        DURATION_ZERO,
    );

    let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);
    let writer_proxy = WriterProxy::new(writer_guid, vec![], vec![]);

    writer.matched_reader_add(reader_proxy);
    reader.matched_writer_add(writer_proxy);

    let cache_change_seq1 = writer.new_change(
        ChangeKind::Alive,
        Some(vec![1, 2, 3]), /*data*/
        None,                /*inline_qos*/
        [0; 16],             /*handle*/
    );

    let cache_change_seq2 = writer.new_change(
        ChangeKind::Alive,
        Some(vec![4, 5, 6]), /*data*/
        None,                /*inline_qos*/
        [0; 16],             /*handle*/
    );

    let cache_change_seq3 = writer.new_change(
        ChangeKind::NotAliveUnregistered,
        None,    /*data*/
        None,    /*inline_qos*/
        [0; 16], /*handle*/
    );

    let cache_change_seq4 = writer.new_change(
        ChangeKind::NotAliveDisposed,
        None,    /*data*/
        None,    /*inline_qos*/
        [0; 16], /*handle*/
    );

    writer.writer_cache().add_change(cache_change_seq1.clone());
    writer.writer_cache().add_change(cache_change_seq2.clone());
    // writer.writer_cache().add_change(cache_change_seq3.clone());
    writer.writer_cache().add_change(cache_change_seq4.clone());

    let writer_message = writer.run(&reader_guid, None).unwrap();

    let mut buf = Vec::new();
    writer_message.compose(&mut buf).unwrap();

    let received_message = RtpsMessage::parse(&buf).unwrap();
    println!("{:?}", received_message);

    reader.run(&writer_guid, Some(&received_message));

    let reader_changes = reader.reader_cache().get_changes();
    assert_eq!(reader_changes.len(), 3);
    assert!(reader_changes.contains(&cache_change_seq1));
    assert!(reader_changes.contains(&cache_change_seq2));
    assert!(!reader_changes.contains(&cache_change_seq3));
    assert!(reader_changes.contains(&cache_change_seq4));
}