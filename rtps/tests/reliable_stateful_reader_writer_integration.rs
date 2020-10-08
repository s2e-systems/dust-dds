use rust_rtps::types::constants::{
    ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
};
use rust_rtps::types::{ChangeKind, TopicKind, GUID, Locator};
use rust_rtps::{
    ReaderProxy, StatefulReader, StatefulWriter, WriterProxy, 
};

use rust_rtps::{MemoryTransport, RtpsMessageReceiver, RtpsMessageSender};

use rust_dds_interface::qos::{DataWriterQos, DataReaderQos};
use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

#[test]
fn reliable_stateful_writer_stateful_reader_data_only() {
    let reader_locator = Locator::new(5, 7400, [2;16]);
    let writer_locator = Locator::new(5, 7400, [1;16]);

    let reader_memory_transport = MemoryTransport::new(reader_locator, vec![]).unwrap();
    let writer_memory_transport = MemoryTransport::new(writer_locator, vec![]).unwrap();

    let guid_prefix = [0; 12];
    let writer_guid = GUID::new(
        guid_prefix,
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    );
    let mut writer_qos = DataWriterQos::default();
    writer_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;

    let writer = StatefulWriter::new(
        writer_guid,
        TopicKind::WithKey,
        &writer_qos                                           
    );
    let reader_guid = GUID::new(
        guid_prefix,
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    );
    let mut reader_qos = DataReaderQos::default();
    reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;
    let reader = StatefulReader::new(
        reader_guid,
        TopicKind::WithKey,
        &reader_qos
    );

    let reader_proxy = ReaderProxy::new(reader_guid, vec![reader_locator], vec![], false, true);
    let writer_proxy = WriterProxy::new(writer_guid, vec![writer_locator], vec![]);

    writer.matched_reader_add(reader_proxy);
    reader.matched_writer_add(writer_proxy);

    let cache_change_seq1 = writer.new_change(
        ChangeKind::Alive,
        Some(vec![1, 2, 3]), 
        None,                
        [0; 16],             
    );

    let cache_change_seq2 = writer.new_change(
        ChangeKind::Alive,
        Some(vec![4, 5, 6]),
        None,               
        [0; 16],            
    );

    let cache_change_seq3 = writer.new_change(
        ChangeKind::NotAliveUnregistered,
        None,   
        None,    
        [0; 16],
    );
    writer.writer_cache().add_change(cache_change_seq1).unwrap();
    writer.writer_cache().add_change(cache_change_seq2).unwrap();
    writer.writer_cache().add_change(cache_change_seq3).unwrap();

    writer.run();
    RtpsMessageSender::send(guid_prefix, &writer_memory_transport, &[&writer]);

    reader_memory_transport.receive_from(&writer_memory_transport);

    RtpsMessageReceiver::receive(guid_prefix, &reader_memory_transport, &[&reader]);

    reader.run();
    reader.run();
    reader.run();

    let reader_changes = reader.reader_cache().changes();
    assert_eq!(reader_changes.len(), writer.writer_cache().changes().len());
    assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 1).is_some());
    assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 2).is_some());
    assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 3).is_some());
}

#[test]
fn reliable_stateful_writer_stateful_reader_data_and_gap() {
    let reader_locator = Locator::new(5, 7400, [2;16]);
    let writer_locator = Locator::new(5, 7400, [1;16]);

    let reader_memory_transport = MemoryTransport::new(reader_locator, vec![]).unwrap();
    let writer_memory_transport = MemoryTransport::new(writer_locator, vec![]).unwrap();

    let guid_prefix = [0; 12];
    let writer_guid = GUID::new(
        guid_prefix,
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    );
    let mut writer_qos = DataWriterQos::default();
    writer_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;
    let writer = StatefulWriter::new(
        writer_guid,
        TopicKind::WithKey,
        &writer_qos
    );
    let reader_guid = GUID::new(
        guid_prefix,
        ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
    );
    let mut reader_qos = DataReaderQos::default();
    reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;
    let reader = StatefulReader::new(
        reader_guid,
        TopicKind::WithKey,
        &reader_qos
    );

    let reader_proxy = ReaderProxy::new(reader_guid, vec![reader_locator], vec![], false, true);
    let writer_proxy = WriterProxy::new(writer_guid, vec![writer_locator], vec![]);

    writer.matched_reader_add(reader_proxy);
    reader.matched_writer_add(writer_proxy);

    let cache_change_seq1 = writer.new_change(
        ChangeKind::Alive,
        Some(vec![1, 2, 3]),
        None,               
        [0; 16],            
    );

    let _cache_change_seq2 = writer.new_change(
        ChangeKind::Alive,
        Some(vec![4, 5, 6]), 
        None,                
        [0; 16],             
    );

    let cache_change_seq3 = writer.new_change(
        ChangeKind::NotAliveUnregistered,
        None,    
        None,    
        [0; 16], 
    );

    writer.writer_cache().add_change(cache_change_seq1).unwrap();
    // writer.writer_cache().add_change(cache_change_seq2.clone());
    writer.writer_cache().add_change(cache_change_seq3).unwrap();

    writer.run();
    RtpsMessageSender::send(guid_prefix, &writer_memory_transport, &[&writer]);
    reader_memory_transport.receive_from(&writer_memory_transport);
    RtpsMessageReceiver::receive(guid_prefix, &reader_memory_transport, &[&reader]);

    reader.run();
    reader.run();
    reader.run();

    let reader_changes = reader.reader_cache().changes();
    assert_eq!(reader_changes.len(), writer.writer_cache().changes().len());
    assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 1).is_some());
    assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 2).is_none());
    assert!(reader_changes.iter().find(|&cc| cc.sequence_number() == 3).is_some());
}