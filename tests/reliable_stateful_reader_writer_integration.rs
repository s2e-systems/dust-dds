// use rust_rtps::behavior::types::constants::DURATION_ZERO;
// use rust_rtps::types::constants::{
//     ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER, ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
// };
// use rust_rtps::types::{ChangeKind, ReliabilityKind, TopicKind, GUID};
// use rust_rtps::{
//     ReaderProxy, StatefulReader, StatefulWriter, WriterProxy, 
// };

// #[test]
// fn reliable_stateful_writer_stateful_reader_data_only() {
//     let guid_prefix = [0; 12];
//     let writer_guid = GUID::new(
//         guid_prefix,
//         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
//     );

//     let writer = StatefulWriter::new(
//         writer_guid,
//         TopicKind::WithKey,
//         ReliabilityKind::Reliable,
//         true,                   
//         DURATION_ZERO,                               
//         DURATION_ZERO,                        
//         DURATION_ZERO,                                            
//     );
//     let reader_guid = GUID::new(
//         [0; 12],
//         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
//     );
//     let reader = StatefulReader::new(
//         reader_guid,
//         TopicKind::WithKey,
//         ReliabilityKind::Reliable,
//         false,
//         DURATION_ZERO,
//     );

//     let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);
//     let writer_proxy = WriterProxy::new(writer_guid, vec![], vec![]);

//     writer.matched_reader_add(reader_proxy);
//     reader.matched_writer_add(writer_proxy);

//     let cache_change_seq1 = writer.new_change(
//         ChangeKind::Alive,
//         Some(vec![1, 2, 3]), 
//         None,                
//         [0; 16],             
//     );

//     let cache_change_seq2 = writer.new_change(
//         ChangeKind::Alive,
//         Some(vec![4, 5, 6]),
//         None,               
//         [0; 16],            
//     );

//     let cache_change_seq3 = writer.new_change(
//         ChangeKind::NotAliveUnregistered,
//         None,   
//         None,    
//         [0; 16],
//     );
//     writer.writer_cache().add_change(cache_change_seq1.clone());
//     writer.writer_cache().add_change(cache_change_seq2.clone());
//     writer.writer_cache().add_change(cache_change_seq3.clone());

//     writer.run();   
//     while let Some(message) = writer.matched_readers()[&reader_guid].pop_send_message() {
//         reader.matched_writers()[&writer_guid].push_receive_message(message);
//     }
//     for _ in 0 .. writer.writer_cache().changes().len() {
//         reader.run();
//     }

//     let reader_changes = reader.reader_cache().changes();
//     assert_eq!(reader_changes.len(), writer.writer_cache().changes().len());
//     assert!(reader_changes.contains(&cache_change_seq1));
//     assert!(reader_changes.contains(&cache_change_seq2));
//     assert!(reader_changes.contains(&cache_change_seq3));
// }

// #[test]
// fn reliable_stateful_writer_stateful_reader_data_and_gap() {
//     let writer_guid = GUID::new(
//         [0; 12],
//         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
//     );
//     let writer = StatefulWriter::new(
//         writer_guid,
//         TopicKind::WithKey,
//         ReliabilityKind::Reliable,                        
//         false,            
//         DURATION_ZERO,    
//         DURATION_ZERO,    
//         DURATION_ZERO,    
//     );
//     let reader_guid = GUID::new(
//         [0; 12],
//         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
//     );
//     let reader = StatefulReader::new(
//         reader_guid,
//         TopicKind::WithKey,
//         ReliabilityKind::Reliable,
//         false,
//         DURATION_ZERO,
//     );

//     let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);
//     let writer_proxy = WriterProxy::new(writer_guid, vec![], vec![]);

//     writer.matched_reader_add(reader_proxy);
//     reader.matched_writer_add(writer_proxy);

//     let cache_change_seq1 = writer.new_change(
//         ChangeKind::Alive,
//         Some(vec![1, 2, 3]),
//         None,               
//         [0; 16],            
//     );

//     let cache_change_seq2 = writer.new_change(
//         ChangeKind::Alive,
//         Some(vec![4, 5, 6]), 
//         None,                
//         [0; 16],             
//     );

//     let cache_change_seq3 = writer.new_change(
//         ChangeKind::NotAliveUnregistered,
//         None,    
//         None,    
//         [0; 16], 
//     );

//     writer.writer_cache().add_change(cache_change_seq1.clone());
//     // writer.writer_cache().add_change(cache_change_seq2.clone());
//     writer.writer_cache().add_change(cache_change_seq3.clone());

//     writer.run();
//     while let Some(message) = writer.matched_readers()[&reader_guid].pop_send_message() {
//         reader.matched_writers()[&writer_guid].push_receive_message(message);
//     }
//     for _ in 0 .. 3 {
//         reader.run();
//     }

//     let reader_changes = reader.reader_cache().changes();
//     assert_eq!(reader_changes.len(), writer.writer_cache().changes().len());
//     assert!(reader_changes.contains(&cache_change_seq1));
//     assert!(!reader_changes.contains(&cache_change_seq2));
//     assert!(reader_changes.contains(&cache_change_seq3));
// }

// #[test]
// fn reliable_stateful_writer_stateful_reader_dropped_messages() {
//     let writer_guid = GUID::new(
//         [0; 12],
//         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
//     );
//     let writer = StatefulWriter::new(
//         writer_guid,
//         TopicKind::WithKey,
//         ReliabilityKind::Reliable,                         
//         false,                               
//         DURATION_ZERO,                       
//         DURATION_ZERO,                     
//         DURATION_ZERO,                     
//     );
//     let reader_guid = GUID::new(
//         [0; 12],
//         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
//     );
//     let reader = StatefulReader::new(
//         reader_guid,
//         TopicKind::WithKey,
//         ReliabilityKind::Reliable,
//         false,
//         DURATION_ZERO,
//     );

//     let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);
//     let writer_proxy = WriterProxy::new(writer_guid, vec![], vec![]);

//     writer.matched_reader_add(reader_proxy);
//     reader.matched_writer_add(writer_proxy);

//     let cache_change_seq1 = writer.new_change(
//         ChangeKind::Alive,
//         Some(vec![1, 2, 3]),
//         None,               
//         [0; 16],            
//     );

//     let cache_change_seq2 = writer.new_change(
//         ChangeKind::Alive,
//         Some(vec![4, 5, 6]), 
//         None,                
//         [0; 16],             
//     );

//     let cache_change_seq3 = writer.new_change(
//         ChangeKind::NotAliveUnregistered,
//         None,       
//         None,   
//         [0; 16],  
//     );

//     writer.writer_cache().add_change(cache_change_seq1.clone());
//     writer.writer_cache().add_change(cache_change_seq2.clone());
//     writer.writer_cache().add_change(cache_change_seq3.clone());

//     // Fill message queue with data messages
//     writer.run();

//     // Send data messages from writer to reader while dropping a message
//     let mut n = 0;
//     while let Some(message) = writer.matched_readers()[&reader_guid].pop_send_message() {        
//         if n != 1 { // Drop 2nd message
//             reader.matched_writers()[&writer_guid].push_receive_message(message);
//         }
//         n += 1;
//     }
    
//     // Fill message queue with heartbeat messages
//     writer.run();

//     // Send heartbeat messages from writer to reader 
//     while let Some(message) = writer.matched_readers()[&reader_guid].pop_send_message() {        
//         reader.matched_writers()[&writer_guid].push_receive_message(message);
//     }

//     // Fill message queue with acknack messages
//     for _ in 0 .. 4 {
//         reader.run();
//     }

//     // Send acknack messages from reader to writer
//     while let Some(message) = reader.matched_writers()[&writer_guid].pop_send_message() {
//         writer.matched_readers()[&reader_guid].push_receive_message(message);
//     }

//     // Writer now fills message queue with data message
//     for _ in 0 .. 2 {
//         writer.run();
//     }

//     // Send again data message from writer to reader 
//     while let Some(message) = writer.matched_readers()[&reader_guid].pop_send_message() {        
//         reader.matched_writers()[&writer_guid].push_receive_message(message);
//     }

//     for _ in 0 .. 3 {
//         reader.run();
//     }

//     let reader_changes = reader.reader_cache().changes();
//     assert_eq!(reader_changes.len(), writer.writer_cache().changes().len());
//     assert!(reader_changes.contains(&cache_change_seq1));
//     assert!(reader_changes.contains(&cache_change_seq2));
//     assert!(reader_changes.contains(&cache_change_seq3));
// }

// #[test]
// fn reliable_stateful_writer_stateful_reader_reordered_data() {
//     let writer_guid = GUID::new(
//         [0; 12],
//         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
//     );
//     let writer = StatefulWriter::new(
//         writer_guid,
//         TopicKind::WithKey,
//         ReliabilityKind::Reliable,
//         false,                               
//         DURATION_ZERO,                       
//         DURATION_ZERO,                       
//         DURATION_ZERO,                       
//     );
//     let reader_guid = GUID::new(
//         [0; 12],
//         ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER,
//     );
//     let reader = StatefulReader::new(
//         reader_guid,
//         TopicKind::WithKey,
//         ReliabilityKind::Reliable,
//         false,
//         DURATION_ZERO,
//     );

//     let reader_proxy = ReaderProxy::new(reader_guid, vec![], vec![], false, true);
//     let writer_proxy = WriterProxy::new(writer_guid, vec![], vec![]);

//     writer.matched_reader_add(reader_proxy);
//     reader.matched_writer_add(writer_proxy);

//     let cache_change_seq1 = writer.new_change(
//         ChangeKind::Alive,
//         Some(vec![1, 2, 3]), 
//         None,                
//         [0; 16],             
//     );

//     let cache_change_seq2 = writer.new_change(
//         ChangeKind::Alive,
//         Some(vec![4, 5, 6]), 
//         None,                
//         [0; 16],             
//     );
    
//     let cache_change_seq3 = writer.new_change(
//         ChangeKind::NotAliveUnregistered,
//         None,    
//         None,   
//         [0; 16], 
//     );

//     writer.writer_cache().add_change(cache_change_seq1.clone());
//     writer.writer_cache().add_change(cache_change_seq2.clone());
//     writer.writer_cache().add_change(cache_change_seq3.clone());

//     // Fill message queue with data messages
//     writer.run();

//     let message1 = writer.matched_readers()[&reader_guid].pop_send_message().unwrap();
//     let message2 = writer.matched_readers()[&reader_guid].pop_send_message().unwrap();
//     let message3 = writer.matched_readers()[&reader_guid].pop_send_message().unwrap();

//     // Send data messages from writer to reader out of order   
//     reader.matched_writers()[&writer_guid].push_receive_message(message3);
//     reader.matched_writers()[&writer_guid].push_receive_message(message1);
//     reader.matched_writers()[&writer_guid].push_receive_message(message2);

//     for _ in 0 .. 3 {
//         reader.run();
//     }

//     let reader_changes = reader.reader_cache().changes();
//     assert_eq!(reader_changes.len(), writer.writer_cache().changes().len());
//     assert!(reader_changes.contains(&cache_change_seq1));
//     assert!(reader_changes.contains(&cache_change_seq2));
//     assert!(reader_changes.contains(&cache_change_seq3));
// }