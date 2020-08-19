use crate::types::{GUID, GuidPrefix, Locator,};
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::structure::stateless_reader::StatelessReader;
use crate::structure::stateful_reader::StatefulReader;
use crate::transport::Transport;

use super::submessage::RtpsSubmessage;

// ////////////////// RTPS Message Receiver

pub trait Receiver {
    fn push_receive_message(&self, source_guid_prefix: GuidPrefix, message: RtpsSubmessage);
    
    fn pop_receive_message(&self) -> Option<(GuidPrefix, RtpsSubmessage)>;
}

pub struct RtpsMessageReceiver{

}

impl RtpsMessageReceiver {

    pub fn receive(participant_guid_prefix: GuidPrefix, transport: &impl Transport, stateless_reader_list: &[&StatelessReader], stateful_reader_list: &[&StatefulReader]) {
        if let Some((message, src_locator)) = transport.read().unwrap() {
            let _source_version = message.header().version();
            let _source_vendor_id = message.header().vendor_id();
            let source_guid_prefix = *message.header().guid_prefix();
            let _dest_guid_prefix = participant_guid_prefix;
            let _unicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
            let _multicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
            let mut _timestamp = None;
            let _message_length = 0;
    
            for submessage in message.take_submessages() {
                match submessage {
                    // Writer to reader messages
                    RtpsSubmessage::Data(data) => RtpsMessageReceiver::receive_reader_submessage(&src_locator, source_guid_prefix, RtpsSubmessage::Data(data),stateless_reader_list,stateful_reader_list),
                    RtpsSubmessage::Gap(gap) => RtpsMessageReceiver::receive_reader_submessage(&src_locator, source_guid_prefix, RtpsSubmessage::Gap(gap),stateless_reader_list,stateful_reader_list),
                    RtpsSubmessage::Heartbeat(heartbeat) => RtpsMessageReceiver::receive_reader_submessage(&src_locator, source_guid_prefix, RtpsSubmessage::Heartbeat(heartbeat),stateless_reader_list,stateful_reader_list),
                    // Reader to writer messages
                    RtpsSubmessage::AckNack(ack_nack) => RtpsMessageReceiver::receive_writer_submessage(source_guid_prefix, RtpsSubmessage::AckNack(ack_nack)),
                    // Receiver status messages
                    RtpsSubmessage::InfoTs(info_ts) => _timestamp = info_ts.time(),
                }
            }
        }    

    }

    fn receive_reader_submessage(source_locator: &Locator, source_guid_prefix: GuidPrefix, message: RtpsSubmessage, stateless_reader_list: &[&StatelessReader], stateful_reader_list: &[&StatefulReader]) {
        let writer_guid = match &message {
            RtpsSubmessage::Data(data) => GUID::new(source_guid_prefix, data.writer_id()),
            RtpsSubmessage::Gap(gap) => GUID::new(source_guid_prefix, gap.writer_id()),
            RtpsSubmessage::Heartbeat(heartbeat) => GUID::new(source_guid_prefix, heartbeat.writer_id()),
            _ => panic!("Unexpected reader message")
        };
    
        for &stateful_reader in stateful_reader_list {
            // Messages are received if they come from a matched writer
            if let Some(writer_proxy) = stateful_reader.matched_writers().get(&writer_guid) {
                writer_proxy.push_receive_message(source_guid_prefix, message);
                return;
            }
        }
    
        let reader_guid_prefix = match &message {
            RtpsSubmessage::Data(data) => data.reader_id(),
            RtpsSubmessage::Gap(gap) => gap.reader_id(),
            RtpsSubmessage::Heartbeat(heartbeat) => heartbeat.reader_id(),
            _ => panic!("Unexpected reader message"),
        };
    
        for &stateless_reader in stateless_reader_list {
            // Messages are received by the stateless reader if they come from the expected source locator and
            // if the destination entity_id matches the reader id or if it is unknown
            if stateless_reader.unicast_locator_list().iter().find(|&loc| loc == source_locator).is_some() ||
                stateless_reader.multicast_locator_list().iter().find(|&loc| loc == source_locator).is_some() {
                
                if stateless_reader.guid().entity_id() == reader_guid_prefix || reader_guid_prefix == ENTITYID_UNKNOWN {
                    stateless_reader.push_receive_message(source_guid_prefix, message);
                    return;
                }
            }
        }
    }
    
    fn receive_writer_submessage(_source_guid_prefix: GuidPrefix, _message: RtpsSubmessage) {
        todo!()
        // let reader_guid = match &message {
        //     WriterReceiveMessage::AckNack(ack_nack) =>  GUID::new(source_guid_prefix, ack_nack.reader_id()),
        // };
    
        // for writer in &self.writer_list {
        //     match writer {
        //         Writer::StatelessWriter(_stateless_writer) => {
        //             // Stateless writers do not receive any message because they are only best effort
        //         },
        //         Writer::StatefulWriter(stateful_writer) => {
        //             if let Some(reader_proxy) = stateful_writer.matched_readers().get(&reader_guid) {
        //                 reader_proxy_received_message(reader_proxy, message);
        //                 break;
        //             }
        //         },
        //     }
        // }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TopicKind, ReliabilityKind};
    use crate::types::constants::{ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_UNKNOWN};
    use crate::transport::memory_transport::MemoryTransport;
    use crate::messages::{Endianness, Data, RtpsMessage, Payload};
    use crate::behavior::types::Duration;
    use crate::structure::stateful_reader::WriterProxy;

    #[test]
    fn stateless_reader_message_receive() {
        let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), None).unwrap();
        let guid_prefix = [1,2,3,4,5,6,8,1,2,3,4,5];

        let src_locator = Locator::new_udpv4(7500, [127,0,0,1]);

        let stateless_reader = StatelessReader::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
            TopicKind::WithKey,
            vec![src_locator],
            vec![],
            false);


        // Run the empty transport and check that nothing happends
        RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateless_reader], &[]);
        assert!(stateless_reader.pop_receive_message().is_none());

        // Send a message to the stateless reader
        let src_guid_prefix = [5,2,3,4,5,6,8,1,2,3,4,5];
        let data = Data::new(Endianness::LittleEndian, ENTITYID_UNKNOWN, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
        let message = RtpsMessage::new(src_guid_prefix, vec![RtpsSubmessage::Data(data)]);
        transport.push_read(message, src_locator);

        RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateless_reader], &[]);

        let expected_data = Data::new(Endianness::LittleEndian, ENTITYID_UNKNOWN, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
        match stateless_reader.pop_receive_message() {
            Some((received_guid_prefix, RtpsSubmessage::Data(data_received))) => {
                assert_eq!(received_guid_prefix, src_guid_prefix);
                assert_eq!(data_received, expected_data);
            },
            _ => panic!("Unexpected message received"),
        };
    }

    #[test]
    fn stateless_reader_message_receive_other_locator() {
        let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), None).unwrap();
        let guid_prefix = [1,2,3,4,5,6,8,1,2,3,4,5];

        let src_locator = Locator::new_udpv4(7500, [127,0,0,1]);

        let stateless_reader = StatelessReader::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
            TopicKind::WithKey,
            vec![src_locator],
            vec![],
            false);

        // Send a message to the stateless reader
        let other_locator = Locator::new_udpv4(7600, [1,1,1,1]);
        let src_guid_prefix = [5,2,3,4,5,6,8,1,2,3,4,5];
        let data = Data::new(Endianness::LittleEndian, ENTITYID_UNKNOWN, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
        let message = RtpsMessage::new(src_guid_prefix, vec![RtpsSubmessage::Data(data)]);
        transport.push_read(message, other_locator);

        RtpsMessageReceiver::receive(guid_prefix, &transport, &[&stateless_reader], &[]);
        assert!(stateless_reader.pop_receive_message().is_none());
    }

    #[test]
    fn stateful_reader_message_receive() {
        let transport = MemoryTransport::new(Locator::new(0,0,[0;16]), None).unwrap();
        let guid_prefix = [1,2,3,4,5,6,8,1,2,3,4,5];

        let stateful_reader = StatefulReader::new(
            GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            false,
            Duration::from_millis(500));

        RtpsMessageReceiver::receive(guid_prefix, &transport, &[], &[&stateful_reader]);

        let remote_guid_prefix = [1;12];
        let remote_guid = GUID::new(remote_guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let src_locator = Locator::new_udpv4(7500, [127,0,0,1]);

        let proxy = WriterProxy::new(remote_guid,vec![src_locator], vec![]);
        stateful_reader.matched_writer_add(proxy);
        
        // Run the empty transport and check that nothing happends
        RtpsMessageReceiver::receive(guid_prefix, &transport, &[], &[&stateful_reader]);
        assert!(stateful_reader.matched_writers().get(&remote_guid).unwrap().pop_receive_message().is_none());

        // Send a message from the matched writer to the reader
        let data = Data::new(Endianness::LittleEndian, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
        let message = RtpsMessage::new(remote_guid_prefix, vec![RtpsSubmessage::Data(data)]);
        transport.push_read(message, src_locator);

        RtpsMessageReceiver::receive(guid_prefix, &transport, &[], &[&stateful_reader]);

        let expected_data = Data::new(Endianness::LittleEndian, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
        match stateful_reader.matched_writers().get(&remote_guid).unwrap().pop_receive_message() {
            Some((_, RtpsSubmessage::Data(data_received))) => {
                assert_eq!(data_received, expected_data);
            },
            _ => panic!("Unexpected message received"),
        };

        // Send a message from an unmatched writer to the reader
        let other_remote_guid_prefix = [10;12];
        let data = Data::new(Endianness::LittleEndian, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, 1, None, Payload::None);
        let message = RtpsMessage::new(other_remote_guid_prefix, vec![RtpsSubmessage::Data(data)]);
        transport.push_read(message, src_locator);

        RtpsMessageReceiver::receive(guid_prefix, &transport, &[], &[&stateful_reader]);
        assert!(stateful_reader.matched_writers().get(&remote_guid).unwrap().pop_receive_message().is_none());
    }    

}