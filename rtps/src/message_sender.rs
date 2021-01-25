use crate::{
    behavior::endpoint_traits::DestinedMessages,
    messages::RtpsMessage,
    transport::Transport,
    types::{
        constants::{PROTOCOL_VERSION_2_4, VENDOR_ID},
        GuidPrefix,
    },
};

pub struct RtpsMessageSender;

impl RtpsMessageSender {
    pub fn send_cache_change_messages(
        participant_guid_prefix: GuidPrefix,
        transport: &dyn Transport,
        destined_messages: Vec<DestinedMessages>,
    ) {
        for destined_message in destined_messages {
            match destined_message {
                DestinedMessages::SingleDestination { locator, messages } => {
                    if messages.len() > 0 {
                        let message = RtpsMessage::new(
                            PROTOCOL_VERSION_2_4,
                            VENDOR_ID,
                            participant_guid_prefix,
                            messages,
                        );
                        transport.write(message, &locator);
                    }
                }
                DestinedMessages::MultiDestination {
                    unicast_locator_list,
                    multicast_locator_list,
                    messages,
                } => {
                    if messages.len() > 0 {
                        let message = RtpsMessage::new(
                            PROTOCOL_VERSION_2_4,
                            VENDOR_ID,
                            participant_guid_prefix,
                            messages,
                        );

                        if !unicast_locator_list.is_empty() {
                            transport.write(message, &unicast_locator_list[0]);
                        } else if !multicast_locator_list.is_empty() {
                            transport.write(message, &multicast_locator_list[0]);
                        }
                    }
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::transport::memory::MemoryTransport;
//     use crate::types::{TopicKind, GUID, ChangeKind, Locator};
//     use crate::types::constants::{
//         ENTITYID_UNKNOWN,
//         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
//         ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
//         ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };
//     use crate::behavior::{StatelessWriter, StatefulWriter, ReaderProxy};

//     use rust_dds_api::qos::DataWriterQos;
//     use rust_dds_api::qos_policy::ReliabilityQosPolicyKind;

//     #[test]
//     fn stateless_writer_single_reader_locator() {
//         let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), vec![]).unwrap();
//         let participant_guid_prefix = [1,2,3,4,5,5,4,3,2,1,1,2];
//         let writer_qos = DataWriterQos::default();
//         let mut stateless_writer_1 = StatelessWriter::new(GUID::new(participant_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER), TopicKind::WithKey, &writer_qos);
//         let reader_locator_1 = Locator::new(-2, 10000, [1;16]);
//         stateless_writer_1.reader_locator_add(reader_locator_1);

//         // Check that nothing is sent to start with
//         RtpsMessageSender::send(participant_guid_prefix, &transport, &mut [&mut stateless_writer_1]);

//         assert_eq!(transport.pop_write(), None);

//         // Add a change to the stateless writer history cache and run the writer
//         let change_1 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
//         stateless_writer_1.writer_cache().add_change(change_1).unwrap();
//         stateless_writer_1.run();

//         RtpsMessageSender::send(participant_guid_prefix, &transport, &mut [&mut stateless_writer_1]);
//         let (message, dst_locator) = transport.pop_write().unwrap();
//         assert_eq!(dst_locator, vec![reader_locator_1]);
//         assert_eq!(message.submessages().len(), 2);
//         match &message.submessages()[0] {
//             RtpsSubmessage::InfoTs(info_ts) => {
//                 assert_eq!(info_ts.time().is_some(), true);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//         match &message.submessages()[1] {
//             RtpsSubmessage::Data(data) => {
//                 assert_eq!(data.reader_id(), ENTITYID_UNKNOWN);
//                 assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//                 assert_eq!(data.writer_sn(), 1);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };

//         // Add two changes to the stateless writer history cache and run the writer
//         let change_2 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
//         let change_3 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
//         stateless_writer_1.writer_cache().add_change(change_2).unwrap();
//         stateless_writer_1.writer_cache().add_change(change_3).unwrap();
//         stateless_writer_1.run();

//         RtpsMessageSender::send(participant_guid_prefix, &transport, &mut [&mut stateless_writer_1]);
//         let (message, dst_locator) = transport.pop_write().unwrap();
//         assert_eq!(dst_locator, vec![reader_locator_1]);
//         assert_eq!(message.submessages().len(), 4);
//         match &message.submessages()[0] {
//             RtpsSubmessage::InfoTs(info_ts) => {
//                 assert_eq!(info_ts.time().is_some(), true);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//         match &message.submessages()[1] {
//             RtpsSubmessage::Data(data) => {
//                 assert_eq!(data.reader_id(), ENTITYID_UNKNOWN);
//                 assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//                 assert_eq!(data.writer_sn(), 2);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//         match &message.submessages()[2] {
//             RtpsSubmessage::InfoTs(info_ts) => {
//                 assert_eq!(info_ts.time().is_some(), true);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//         match &message.submessages()[3] {
//             RtpsSubmessage::Data(data) => {
//                 assert_eq!(data.reader_id(), ENTITYID_UNKNOWN);
//                 assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//                 assert_eq!(data.writer_sn(), 3);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };

//         // Add two new changes but only one to the stateless writer history cache and run the writer
//         let _change_4 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
//         let change_5 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
//         stateless_writer_1.writer_cache().add_change(change_5).unwrap();
//         stateless_writer_1.run();

//         RtpsMessageSender::send(participant_guid_prefix, &transport, &mut [&mut stateless_writer_1]);
//         let (message, dst_locator) = transport.pop_write().unwrap();
//         assert_eq!(dst_locator, vec![reader_locator_1]);
//         assert_eq!(message.submessages().len(), 4);
//         match &message.submessages()[0] {
//             RtpsSubmessage::InfoTs(info_ts) => {
//                 assert_eq!(info_ts.time().is_some(), true);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//         match &message.submessages()[1] {
//             RtpsSubmessage::Gap(gap) => {
//                 assert_eq!(gap.reader_id(), ENTITYID_UNKNOWN);
//                 assert_eq!(gap.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//         match &message.submessages()[2] {
//             RtpsSubmessage::InfoTs(info_ts) => {
//                 assert_eq!(info_ts.time().is_some(), true);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//         match &message.submessages()[3] {
//             RtpsSubmessage::Data(data) => {
//                 assert_eq!(data.reader_id(), ENTITYID_UNKNOWN);
//                 assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//                 assert_eq!(data.writer_sn(), 5);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//     }

//     #[test]
//     fn multiple_stateless_writers_multiple_reader_locators() {
//         let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), vec![]).unwrap();
//         let participant_guid_prefix = [1,2,3,4,5,5,4,3,2,1,1,2];

//         let reader_locator_1 = Locator::new(-2, 10000, [1;16]);
//         let reader_locator_2 = Locator::new(-2, 2000, [2;16]);
//         let reader_locator_3 = Locator::new(-2, 300, [3;16]);

//         let writer_qos = DataWriterQos::default();

//         let mut stateless_writer_1 = StatelessWriter::new(GUID::new(participant_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER), TopicKind::WithKey, &writer_qos);
//         let mut stateless_writer_2 = StatelessWriter::new(GUID::new(participant_guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER), TopicKind::WithKey, &writer_qos);

//         RtpsMessageSender::send(participant_guid_prefix, &transport, &mut [&mut stateless_writer_1, &mut stateless_writer_2]);

//         stateless_writer_1.reader_locator_add(reader_locator_1);

//         stateless_writer_2.reader_locator_add(reader_locator_1);
//         stateless_writer_2.reader_locator_add(reader_locator_2);
//         stateless_writer_2.reader_locator_add(reader_locator_3);

//         let change_1_1 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
//         let change_1_2 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
//         stateless_writer_1.writer_cache().add_change(change_1_1).unwrap();
//         stateless_writer_1.writer_cache().add_change(change_1_2).unwrap();

//         let _change_2_1 = stateless_writer_2.new_change(ChangeKind::Alive, Some(vec![1,2,4]), None, [12;16]);
//         let change_2_2 = stateless_writer_2.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [12;16]);
//         stateless_writer_2.writer_cache().add_change(change_2_2).unwrap();

//         stateless_writer_1.run();
//         stateless_writer_2.run();

//         RtpsMessageSender::send(participant_guid_prefix, &transport, &mut [&mut stateless_writer_1, &mut stateless_writer_2]);

//         // The order at which the messages are sent is not predictable so collect eveything in a vector and test from there onwards
//         let mut sent_messages = Vec::new();
//         sent_messages.push(transport.pop_write().unwrap());
//         sent_messages.push(transport.pop_write().unwrap());
//         sent_messages.push(transport.pop_write().unwrap());
//         sent_messages.push(transport.pop_write().unwrap());

//         assert_eq!(sent_messages.iter().filter(|&(_, locator)| locator==&vec![reader_locator_1]).count(), 2);
//         assert_eq!(sent_messages.iter().filter(|&(_, locator)| locator==&vec![reader_locator_2]).count(), 1);
//         assert_eq!(sent_messages.iter().filter(|&(_, locator)| locator==&vec![reader_locator_3]).count(), 1);

//         assert!(transport.pop_write().is_none());
//     }

//     #[test]
//     fn stateful_writer_multiple_reader_locators() {
//         let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), vec![]).unwrap();
//         let participant_guid_prefix = [1,2,3,4,5,5,4,3,2,1,1,2];

//         let mut writer_qos = DataWriterQos::default();
//         writer_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;

//         let mut stateful_writer_1 = StatefulWriter::new(
//             GUID::new(participant_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
//             TopicKind::WithKey,
//             &writer_qos
//         );

//         let reader_guid_prefix = [5;12];
//         let unicast_locator_1 = Locator::new(-2, 10000, [1;16]);
//         let unicast_locator_2 = Locator::new(-2, 200, [2;16]);
//         let multicast_locator = Locator::new(-2, 5, [10;16]);
//         let reader_proxy_1 = ReaderProxy::new(
//             GUID::new(reader_guid_prefix,ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
//             vec![unicast_locator_1, unicast_locator_2],
//             vec![multicast_locator],
//             false,
//             true);

//         stateful_writer_1.matched_reader_add(reader_proxy_1);

//         // Check that nothing is sent to start with
//         RtpsMessageSender::send(participant_guid_prefix, &transport, &mut [&mut stateful_writer_1]);
//         assert_eq!(transport.pop_write(), None);

//         // Add a change to the stateless writer history cache and run the writer
//         let change_1 = stateful_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
//         stateful_writer_1.writer_cache().add_change(change_1).unwrap();
//         stateful_writer_1.run();

//         RtpsMessageSender::send(participant_guid_prefix, &transport, &mut [&mut stateful_writer_1]);
//         let (message, dst_locator) = transport.pop_write().unwrap();
//         assert_eq!(dst_locator, vec![unicast_locator_1, unicast_locator_2,  multicast_locator ]);
//         assert_eq!(message.submessages().len(), 2);
//         match &message.submessages()[0] {
//             RtpsSubmessage::InfoTs(info_ts) => {
//                 assert_eq!(info_ts.time().is_some(), true);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//         match &message.submessages()[1] {
//             RtpsSubmessage::Data(data) => {
//                 assert_eq!(data.reader_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
//                 assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
//                 assert_eq!(data.writer_sn(), 1);
//             },
//             _ => panic!("Unexpected submessage type received"),
//         };
//     }

// }
