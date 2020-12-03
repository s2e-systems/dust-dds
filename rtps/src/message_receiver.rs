use crate::types::{GuidPrefix, Locator,};
use crate::transport::Transport;
use crate::messages::submessages::RtpsSubmessage;
use crate::behavior::endpoint_traits::{CacheChangeReceiver,AcknowldegmentReceiver};

pub struct RtpsMessageReceiver;

impl RtpsMessageReceiver {
    pub fn receive(
        participant_guid_prefix: GuidPrefix,
        transport: &dyn Transport,
        cache_change_receiver_list: &mut [&mut dyn CacheChangeReceiver],
        acknoledgment_receiver_list: &mut Vec<&mut dyn AcknowldegmentReceiver>) {

        if let Some((message, _src_locator)) = transport.read().unwrap() {
            let _source_version = message.header().version();
            let _source_vendor_id = message.header().vendor_id();
            let source_guid_prefix = message.header().guid_prefix();
            let _dest_guid_prefix = participant_guid_prefix;
            let _unicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
            let _multicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
            let mut _timestamp = None;
            let _message_length = 0;
    
            for submessage in message.take_submessages() {
                if submessage.is_entity_submessage() {
                    let mut optional_submessage = Some(submessage);
                    for receiver in cache_change_receiver_list.iter_mut() {
                        receiver.try_process_message(source_guid_prefix, &mut optional_submessage);
                    }
                    for receiver in acknoledgment_receiver_list.iter_mut() {
                        receiver.try_process_message(source_guid_prefix, &mut optional_submessage);
                    }
                } else if submessage.is_interpreter_submessage(){
                    match submessage {
                        RtpsSubmessage::InfoTs(info_ts) => _timestamp = info_ts.time(),
                        _ => panic!("Unexpected interpreter submessage"),
                    };
                }
            }
        }    
    }
}


#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::TopicKind;
    // use crate::types::constants::{ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_UNKNOWN, PROTOCOL_VERSION_2_4, VENDOR_ID};
    // use crate::messages::Endianness;
    // use crate::transport::memory::MemoryTransport;
    // use crate::messages::RtpsMessage;
    // use crate::messages::submessages::Data;
    // use crate::messages::submessages::data_submessage::Payload;
    // use crate::behavior::{StatelessReader, StatefulReader, WriterProxy};

    // use rust_dds_api::qos::DataReaderQos;
    // use rust_dds_api::qos_policy::ReliabilityQosPolicyKind;

    // #[test]
    // fn stateless_reader_message_receive() {
    //     let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), vec![]).unwrap();
    //     let guid_prefix = [1,2,3,4,5,6,8,1,2,3,4,5];

    //     let src_locator = Locator::new_udpv4(7500, [127,0,0,1]);

    //     let reader_qos = DataReaderQos::default();
        
    //     let stateless_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
    //     let stateless_reader = StatelessReader::new(
    //         stateless_reader_guid,
    //         TopicKind::WithKey,
    //         vec![src_locator],
    //         vec![],
    //         &reader_qos);


    //     // Run the empty transport and check that nothing happends
    //     RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateless_reader]);
    //     assert!(stateless_reader.pop_receive_message(&stateless_reader_guid).is_none());

    //     // Send a message to the stateless reader
    //     let src_guid_prefix = [5,2,3,4,5,6,8,1,2,3,4,5];
    //     let data = Data::new(Endianness::LittleEndian, ENTITYID_UNKNOWN, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
    //     let message = RtpsMessage::new(PROTOCOL_VERSION_2_4, VENDOR_ID, src_guid_prefix, vec![RtpsSubmessage::Data(data)]);
    //     transport.push_read(message, src_locator);

    //     RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateless_reader]);

    //     let expected_data = Data::new(Endianness::LittleEndian, ENTITYID_UNKNOWN, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
    //     match stateless_reader.pop_receive_message(&stateless_reader_guid) {
    //         Some((received_guid_prefix, RtpsSubmessage::Data(data_received))) => {
    //             assert_eq!(received_guid_prefix, src_guid_prefix);
    //             assert_eq!(data_received, expected_data);
    //         },
    //         _ => panic!("Unexpected message received"),
    //     };
    // }

    // #[test]
    // fn stateless_reader_message_receive_other_locator() {
    //     let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), vec![]).unwrap();
    //     let guid_prefix = [1,2,3,4,5,6,8,1,2,3,4,5];

    //     let src_locator = Locator::new_udpv4(7500, [127,0,0,1]);

    //     let reader_qos = DataReaderQos::default();

    //     let stateless_reader_guid = GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
    //     let stateless_reader = StatelessReader::new(
    //         stateless_reader_guid,
    //         TopicKind::WithKey,
    //         vec![src_locator],
    //         vec![],
    //         &reader_qos);

    //     // Send a message to the stateless reader
    //     let other_locator = Locator::new_udpv4(7600, [1,1,1,1]);
    //     let src_guid_prefix = [5,2,3,4,5,6,8,1,2,3,4,5];
    //     let data = Data::new(Endianness::LittleEndian, ENTITYID_UNKNOWN, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
    //     let message = RtpsMessage::new(PROTOCOL_VERSION_2_4, VENDOR_ID, src_guid_prefix, vec![RtpsSubmessage::Data(data)]);
    //     transport.push_read(message, other_locator);

    //     RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateless_reader]);
    //     assert!(stateless_reader.pop_receive_message(&stateless_reader_guid).is_none());
    // }

    // #[test]
    // fn stateful_reader_message_receive() {
    //     let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), vec![]).unwrap();
    //     let guid_prefix = [1,2,3,4,5,6,8,1,2,3,4,5];

    //     let mut reader_qos = DataReaderQos::default();
    //     reader_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;

    //     let stateful_reader = StatefulReader::new(
    //         GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
    //         TopicKind::WithKey,
    //         &reader_qos);

    //     RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateful_reader]);

    //     let remote_guid_prefix = [1;12];
    //     let remote_guid = GUID::new(remote_guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
    //     let src_locator = Locator::new_udpv4(7500, [127,0,0,1]);

    //     let proxy = WriterProxy::new(remote_guid,vec![src_locator], vec![]);
    //     stateful_reader.matched_writer_add(proxy);
        
    //     // Run the empty transport and check that nothing happends
    //     RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateful_reader]);
    //     assert!(stateful_reader.pop_receive_message(&remote_guid).is_none());

    //     // Send a message from the matched writer to the reader
    //     let data = Data::new(Endianness::LittleEndian, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
    //     let message = RtpsMessage::new(PROTOCOL_VERSION_2_4, VENDOR_ID, remote_guid_prefix, vec![RtpsSubmessage::Data(data)]);
    //     transport.push_read(message, src_locator);

    //     RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateful_reader]);

    //     let expected_data = Data::new(Endianness::LittleEndian, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
    //     match stateful_reader.pop_receive_message(&remote_guid) {
    //         Some((_, RtpsSubmessage::Data(data_received))) => {
    //             assert_eq!(data_received, expected_data);
    //         },
    //         _ => panic!("Unexpected message received"),
    //     };

    //     // Send a message from an unmatched writer to the reader
    //     let other_remote_guid_prefix = [10;12];
    //     let data = Data::new(Endianness::LittleEndian, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
    //     let message = RtpsMessage::new(PROTOCOL_VERSION_2_4, VENDOR_ID, other_remote_guid_prefix, vec![RtpsSubmessage::Data(data)]);
    //     transport.push_read(message, src_locator);

    //     RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateful_reader]);
    //     assert!(stateful_reader.pop_receive_message(&remote_guid).is_none());
    // }    

}