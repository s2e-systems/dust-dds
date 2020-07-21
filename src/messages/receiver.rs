use crate::types::{GUID, GuidPrefix, Locator};
use crate::stateless_reader::StatelessReader;
use crate::stateless_writer::StatelessWriter;
use crate::stateful_reader::{StatefulReader, WriterProxy};
use crate::stateful_writer::{StatefulWriter, ReaderProxy};
use super::submessage::RtpsSubmessage;
use super::{Data, Gap, Heartbeat, AckNack};
use super::message::RtpsMessage;

pub enum Reader<'a> {
    StatelessReader(&'a StatelessReader),
    StatefulReader(&'a StatefulReader),
}

pub enum Writer<'a> {
    StatelessWriter(&'a StatelessWriter),
    StatefulWriter(&'a StatefulWriter),
}

// Messages received by the reader. Which are the same as the ones sent by the writer
pub enum ReaderReceiveMessage {
    Data(Data),
    Gap(Gap),
    Heartbeat(Heartbeat),
}

pub type WriterSendMessage = ReaderReceiveMessage;

// Messages received by the writer. Which are the same as the ones sent by the reader
pub enum WriterReceiveMessage {
    AckNack(AckNack)
}

pub type ReaderSendMessage = WriterReceiveMessage;

pub struct RtpsMessageReceiver<'a>{
    participant_guid_prefix: GuidPrefix,
    locator: Locator,
    // socket: Transport
    reader_list: Vec<Reader<'a>>,
    writer_list: Vec<Writer<'a>>,
}

impl<'a> RtpsMessageReceiver<'a> {
    pub fn new(participant_guid_prefix: GuidPrefix, locator: Locator, reader_list: Vec<Reader<'a>>, writer_list: Vec<Writer<'a>>) -> Self {
        Self {
            participant_guid_prefix,
            locator,
            reader_list,
            writer_list,
        }
    }

    pub fn receive(&self, message: RtpsMessage) {
        let _source_version = message.header().version();
        let _source_vendor_id = message.header().vendor_id();
        let source_guid_prefix = *message.header().guid_prefix();
        let _dest_guid_prefix = self.participant_guid_prefix;
        let _unicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
        let _multicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
        let mut timestamp = None;
        let _message_length = 0;
        
        let source_locator = Locator::new(0,0, [0;16]);

        for submessage in message.take_submessages() {
            match submessage {
                // Writer to reader messages
                RtpsSubmessage::Data(data) => self.receive_reader_submessage(&source_locator, source_guid_prefix, ReaderReceiveMessage::Data(data)),
                RtpsSubmessage::Gap(gap) => self.receive_reader_submessage(&source_locator, source_guid_prefix, ReaderReceiveMessage::Gap(gap)),
                RtpsSubmessage::Heartbeat(heartbeat) => self.receive_reader_submessage(&source_locator, source_guid_prefix, ReaderReceiveMessage::Heartbeat(heartbeat)),
                // Reader to writer messages
                RtpsSubmessage::AckNack(ack_nack) => self.receive_writer_submessage(source_guid_prefix, WriterReceiveMessage::AckNack(ack_nack)),
                // Receiver status messages
                RtpsSubmessage::InfoTs(info_ts) => timestamp = info_ts.time(),
            }
        }
    }

    fn receive_reader_submessage(&self, source_locator: &Locator, source_guid_prefix: GuidPrefix, message: ReaderReceiveMessage) {
        let writer_guid = match &message {
            ReaderReceiveMessage::Data(data) => GUID::new(source_guid_prefix, data.writer_id()),
            ReaderReceiveMessage::Gap(gap) => GUID::new(source_guid_prefix, gap.writer_id()),
            ReaderReceiveMessage::Heartbeat(heartbeat) => GUID::new(source_guid_prefix, heartbeat.writer_id()),
        };

        for reader in &self.reader_list {
            match reader {
                Reader::StatelessReader(stateless_reader) => {
                    if stateless_reader.unicast_locator_list().iter().find(|&loc| loc == source_locator).is_some() ||
                       stateless_reader.multicast_locator_list().iter().find(|&loc| loc == source_locator).is_some() {
                        RtpsMessageReceiver::stateless_reader_received_message(stateless_reader, source_guid_prefix, message);
                        break;
                    }
                },
                Reader::StatefulReader(stateful_reader) => {
                    if let Some(writer_proxy) = stateful_reader.matched_writers().get(&writer_guid) {
                        RtpsMessageReceiver::writer_proxy_received_message(writer_proxy, message);
                        break;
                    }
                },
            }
        }
    }

    fn receive_writer_submessage(&self, source_guid_prefix: GuidPrefix, message: WriterReceiveMessage) {
        let reader_guid = match &message {
            WriterReceiveMessage::AckNack(ack_nack) =>  GUID::new(source_guid_prefix, ack_nack.reader_id()),
        };

        for writer in &self.writer_list {
            match writer {
                Writer::StatelessWriter(_stateless_writer) => {
                    // Stateless writers do not receive any message because they are only best effort
                },
                Writer::StatefulWriter(stateful_writer) => {
                    if let Some(reader_proxy) = stateful_writer.matched_reader_lookup(&reader_guid) {
                        RtpsMessageReceiver::reader_proxy_received_message(reader_proxy, message);
                        break;
                    }
                },
            }
        }
    }

    fn reader_proxy_received_message(_reader_proxy: &ReaderProxy, _message: WriterReceiveMessage) {
        todo!()
    }

    fn writer_proxy_received_message(writer_proxy: &WriterProxy, message: ReaderReceiveMessage) {
        writer_proxy.push_received_message(message)
    }

    fn stateless_reader_received_message(stateless_reader: &StatelessReader, source_guid_prefix: GuidPrefix, message: ReaderReceiveMessage) {
        stateless_reader.push_receive_message(source_guid_prefix, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{InfoTs, Data};
    use crate::messages::Endianness;
    use crate::messages::types::Time;
    use crate::messages::data_submessage::Payload;
    use crate::types::{GUID, EntityId, EntityKind, TopicKind, ReliabilityKind};
    use crate::stateful_reader::WriterProxy;
    use crate::behavior::types::Duration;

    #[test]
    fn receive_infots_data_message() {
        let guid_prefix = [1;12];
        let guid = GUID::new(guid_prefix, EntityId::new([1,2,3], EntityKind::UserDefinedReaderWithKey));
        let mut reader1 = StatefulReader::new(
            guid,
            TopicKind::WithKey, 
        ReliabilityKind::BestEffort,
        vec![],
        vec![],
        false,
        Duration::from_millis(500));

        let proxy1 = WriterProxy::new(
            GUID::new([2;12], EntityId::new([1,2,3], EntityKind::UserDefinedWriterWithKey)),
            vec![],
            vec![]);

        reader1.matched_writer_add(proxy1);

        let receiver = RtpsMessageReceiver::new(guid_prefix, Locator::new(0,0, [0;16]), vec![Reader::StatefulReader(&reader1)], vec![]);

        let info_ts = InfoTs::new(Some(Time::new(100,100)), Endianness::LittleEndian);
        let data = Data::new(Endianness::LittleEndian,
            EntityId::new([1,2,3], EntityKind::UserDefinedReaderWithKey),
            EntityId::new([1,2,3], EntityKind::UserDefinedWriterWithKey),
            1, None, Payload::Data(vec![1,2,3,4]));
        let message = RtpsMessage::new([2;12],vec![RtpsSubmessage::InfoTs(info_ts), RtpsSubmessage::Data(data)]);

        receiver.receive(message);
    }
}