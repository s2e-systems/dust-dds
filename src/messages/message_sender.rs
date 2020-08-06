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

pub fn rtps_message_sender(transport: &mut Transport, participant_guid_prefix: GuidPrefix, stateless_writer_list: &[&StatelessWriter]) {
    for stateless_writer in stateless_writer_list {
        let reader_locators = stateless_writer.reader_locators();
        for (locator, reader_locator) in reader_locators.iter() {
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
                transport.write(rtps_message, *locator);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}