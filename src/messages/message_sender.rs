use crate::types::{GUID, GuidPrefix, Locator,};
use crate::stateless_reader::StatelessReader;
use crate::stateless_writer::StatelessWriter;
use crate::stateful_reader::{StatefulReader, WriterProxy};
use crate::stateful_writer::{StatefulWriter, ReaderProxy};
use crate::transport::Transport;


use super::submessage::RtpsSubmessage;
use super::{Data, Gap, Heartbeat, AckNack, InfoTs, Endianness, UdpPsmMapping};
use super::message::{RtpsMessage};
use super::types::Time;
use super::message_receiver::ReaderReceiveMessage;

// Messages received by the writer. Which are the same as the ones sent by the reader
pub enum WriterReceiveMessage {
    AckNack(AckNack)
}

pub type WriterSendMessage = ReaderReceiveMessage;

pub fn rtps_message_sender(transport: &impl Transport, participant_guid_prefix: GuidPrefix, stateless_writer_list: &[&StatelessWriter]) {
    for stateless_writer in stateless_writer_list {
        let reader_locators = stateless_writer.reader_locators();
        for (&locator, reader_locator) in reader_locators.iter() {
            let mut submessage = Vec::new();
            while let Some(message) = reader_locator.pop_send_message() {
                submessage.push(RtpsSubmessage::InfoTs(InfoTs::new(Some(Time::now()), Endianness::LittleEndian)));
                match message {
                    WriterSendMessage::Data(data) => submessage.push(RtpsSubmessage::Data(data)),
                    WriterSendMessage::Gap(gap) => submessage.push(RtpsSubmessage::Gap(gap)),
                    WriterSendMessage::Heartbeat(_) => panic!("Heartbeat not expected from stateless writer"),
                };
            };

            if !submessage.is_empty() {
                let rtps_message = RtpsMessage::new(participant_guid_prefix, submessage);
                transport.write(rtps_message, locator);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport_stub::StubTransport;
    use crate::types::{TopicKind, GUID, ChangeKind};
    use crate::types::constants::{
        ENTITYID_UNKNOWN,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };

    #[test]
    fn stateless_writer_single_reader_locator() {
        let transport = StubTransport::new();
        let participant_guid_prefix = [1,2,3,4,5,5,4,3,2,1,1,2];

        let stateless_writer_1 = StatelessWriter::new(GUID::new(participant_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER), TopicKind::WithKey);
        let reader_locator_1 = Locator::new(-2, 10000, [1;16]);
        stateless_writer_1.reader_locator_add(reader_locator_1);

        // Check that nothing is sent to start with
        rtps_message_sender(&transport, participant_guid_prefix, &[&stateless_writer_1]);
        
        assert_eq!(transport.pop_write(), None);

        // Add a change to the stateless writer history cache and run the writer
        let change_1 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
        stateless_writer_1.history_cache().add_change(change_1);
        stateless_writer_1.run();

        rtps_message_sender(&transport, participant_guid_prefix, &[&stateless_writer_1]);
        let (message, dst_locator) = transport.pop_write().unwrap();
        assert_eq!(dst_locator, reader_locator_1);
        assert_eq!(message.submessages().len(), 2);
        match &message.submessages()[0] {
            RtpsSubmessage::InfoTs(info_ts) => {
                assert_eq!(info_ts.time().is_some(), true);
            },
            _ => panic!("Unexpected submessage type received"),
        };
        match &message.submessages()[1] {
            RtpsSubmessage::Data(data) => {
                assert_eq!(data.reader_id(), ENTITYID_UNKNOWN);
                assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
                assert_eq!(data.writer_sn(), 1);
            },
            _ => panic!("Unexpected submessage type received"),
        };

        // Add two changes to the stateless writer history cache and run the writer
        let change_2 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
        let change_3 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
        stateless_writer_1.history_cache().add_change(change_2);
        stateless_writer_1.history_cache().add_change(change_3);
        stateless_writer_1.run();

        rtps_message_sender(&transport, participant_guid_prefix, &[&stateless_writer_1]);
        let (message, dst_locator) = transport.pop_write().unwrap();
        assert_eq!(dst_locator, reader_locator_1);
        assert_eq!(message.submessages().len(), 4);
        match &message.submessages()[0] {
            RtpsSubmessage::InfoTs(info_ts) => {
                assert_eq!(info_ts.time().is_some(), true);
            },
            _ => panic!("Unexpected submessage type received"),
        };
        match &message.submessages()[1] {
            RtpsSubmessage::Data(data) => {
                assert_eq!(data.reader_id(), ENTITYID_UNKNOWN);
                assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
                assert_eq!(data.writer_sn(), 2);
            },
            _ => panic!("Unexpected submessage type received"),
        };
        match &message.submessages()[2] {
            RtpsSubmessage::InfoTs(info_ts) => {
                assert_eq!(info_ts.time().is_some(), true);
            },
            _ => panic!("Unexpected submessage type received"),
        };
        match &message.submessages()[3] {
            RtpsSubmessage::Data(data) => {
                assert_eq!(data.reader_id(), ENTITYID_UNKNOWN);
                assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
                assert_eq!(data.writer_sn(), 3);
            },
            _ => panic!("Unexpected submessage type received"),
        };

        // Add two new changes but only one to the stateless writer history cache and run the writer
        let _change_4 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
        let change_5 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
        stateless_writer_1.history_cache().add_change(change_5);
        stateless_writer_1.run();

        rtps_message_sender(&transport, participant_guid_prefix, &[&stateless_writer_1]);
        let (message, dst_locator) = transport.pop_write().unwrap();
        assert_eq!(dst_locator, reader_locator_1);
        assert_eq!(message.submessages().len(), 4);
        match &message.submessages()[0] {
            RtpsSubmessage::InfoTs(info_ts) => {
                assert_eq!(info_ts.time().is_some(), true);
            },
            _ => panic!("Unexpected submessage type received"),
        };
        match &message.submessages()[1] {
            RtpsSubmessage::Gap(gap) => {
                assert_eq!(gap.reader_id(), ENTITYID_UNKNOWN);
                assert_eq!(gap.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
            },
            _ => panic!("Unexpected submessage type received"),
        };
        match &message.submessages()[2] {
            RtpsSubmessage::InfoTs(info_ts) => {
                assert_eq!(info_ts.time().is_some(), true);
            },
            _ => panic!("Unexpected submessage type received"),
        };
        match &message.submessages()[3] {
            RtpsSubmessage::Data(data) => {
                assert_eq!(data.reader_id(), ENTITYID_UNKNOWN);
                assert_eq!(data.writer_id(), ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
                assert_eq!(data.writer_sn(), 5);
            },
            _ => panic!("Unexpected submessage type received"),
        };
    }

    #[test]
    fn multiple_stateless_writers_multiple_reader_locators() {
        let transport = StubTransport::new();
        let participant_guid_prefix = [1,2,3,4,5,5,4,3,2,1,1,2];

        let reader_locator_1 = Locator::new(-2, 10000, [1;16]);
        let reader_locator_2 = Locator::new(-2, 2000, [2;16]);
        let reader_locator_3 = Locator::new(-2, 300, [3;16]);

        let stateless_writer_1 = StatelessWriter::new(GUID::new(participant_guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER), TopicKind::WithKey);
        let stateless_writer_2 = StatelessWriter::new(GUID::new(participant_guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER), TopicKind::WithKey);
        
        stateless_writer_1.reader_locator_add(reader_locator_1);

        stateless_writer_2.reader_locator_add(reader_locator_1);
        stateless_writer_2.reader_locator_add(reader_locator_2);
        stateless_writer_2.reader_locator_add(reader_locator_3);

        let change_1_1 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
        let change_1_2 = stateless_writer_1.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [5;16]);
        stateless_writer_1.history_cache().add_change(change_1_1);
        stateless_writer_1.history_cache().add_change(change_1_2);

        let _change_2_1 = stateless_writer_2.new_change(ChangeKind::Alive, Some(vec![1,2,4]), None, [12;16]);
        let change_2_2 = stateless_writer_2.new_change(ChangeKind::Alive, Some(vec![1,2,3,4]), None, [12;16]);
        stateless_writer_2.history_cache().add_change(change_2_2);

        stateless_writer_1.run();
        stateless_writer_2.run();

        rtps_message_sender(&transport, participant_guid_prefix, &[&stateless_writer_1, &stateless_writer_2]);

        // The order at which the messages are sent is not predictable so collect eveything in a vector and test from there onwards
        let mut sent_messages = Vec::new();
        sent_messages.push(transport.pop_write().unwrap());
        sent_messages.push(transport.pop_write().unwrap());
        sent_messages.push(transport.pop_write().unwrap());
        sent_messages.push(transport.pop_write().unwrap());

        assert_eq!(sent_messages.iter().filter(|&(_, locator)| locator==&reader_locator_1).count(), 2);
        assert_eq!(sent_messages.iter().filter(|&(_, locator)| locator==&reader_locator_2).count(), 1);
        assert_eq!(sent_messages.iter().filter(|&(_, locator)| locator==&reader_locator_3).count(), 1);

        assert!(transport.pop_write().is_none());
        
    }


}