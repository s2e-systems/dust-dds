use crate::types::{GUID, GuidPrefix, Locator};
use crate::stateless_reader::StatelessReader;
use crate::stateless_writer::StatelessWriter;
use crate::stateful_reader::StatefulReader;
use crate::stateful_writer::StatefulWriter;
use super::types::constants::TIME_INVALID;
use super::submessage::RtpsSubmessage;
use super::{Data, Gap, Heartbeat};
use super::message::RtpsMessage;

pub enum Reader<'a> {
    StatelessReader(&'a StatelessReader),
    StatefulReader(&'a StatefulReader),
}

pub enum Writer<'a> {
    StatelessWriter(&'a StatelessWriter),
    StatefulWriter(&'a StatefulWriter),
}

enum ReaderReceiveMessage {
    Data(Data),
    Gap(Gap),
    Heartbeat(Heartbeat),
}

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
        let dest_guid_prefix = self.participant_guid_prefix;
        let _unicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
        let _multicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
        let mut timestamp = None;
        let _message_length = 0;
        
        let source_locator = Locator::new(0,0, [0;16]);

        for submessage in message.take_submessages() {
            match submessage {
                // Writer to reader messages
                RtpsSubmessage::Data(data) => self.receive_reader_message(&source_locator, source_guid_prefix, ReaderReceiveMessage::Data(data)),
                RtpsSubmessage::Gap(gap) => self.receive_reader_message(&source_locator, source_guid_prefix, ReaderReceiveMessage::Gap(gap)),
                RtpsSubmessage::Heartbeat(heartbeat) => self.receive_reader_message(&source_locator, source_guid_prefix, ReaderReceiveMessage::Heartbeat(heartbeat)),
                // Reader to writer messages
                RtpsSubmessage::AckNack(_ack_nack) => todo!(),
                // Receiver status messages
                RtpsSubmessage::InfoTs(info_ts) => timestamp = info_ts.time(),
            }
        }
    }

    fn receive_reader_message(&self, source_locator: &Locator, source_guid_prefix: GuidPrefix, message: ReaderReceiveMessage) {
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
                        todo!();
                        break;
                    }
                },
                Reader::StatefulReader(stateful_reader) => {
                    if let Some(_writer_proxy) = stateful_reader.matched_writer_lookup(&writer_guid) {
                        todo!();
                        break;
                    }
                },
            }
        }
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
    use crate::stateless_reader::StatelessReader;
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