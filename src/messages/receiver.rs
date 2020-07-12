use crate::types::{GUID, GuidPrefix, Locator};
use crate::stateless_reader::StatelessReader;
use crate::stateful_reader::StatefulReader;
use super::types::constants::TIME_INVALID;
use super::submessage::RtpsSubmessage;
use super::message::RtpsMessage;

pub struct RtpsMessageReceiver<'a>{
    participant_guid_prefix: GuidPrefix,
    // spdp_stateless_reader: &'a StatelessReader,
    stateful_reader_list: Vec<&'a StatefulReader>,

}

impl<'a> RtpsMessageReceiver<'a> {
    pub fn new(participant_guid_prefix: GuidPrefix, stateful_reader_list: &[&'a StatefulReader]) -> Self {
        Self {
            participant_guid_prefix,
            stateful_reader_list: stateful_reader_list.to_vec(),
        }
    }

    pub fn receive(&self, message: RtpsMessage) {
        let _source_version = message.header().version();
        let _source_vendor_id = message.header().vendor_id();
        let source_guid_prefix = *message.header().guid_prefix();
        let dest_guid_prefix = self.participant_guid_prefix; // TODO: This is ourselves
        let _unicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
        let _multicast_reply_locator_list = vec![Locator::new(0,0,[0;16])];
        let mut timestamp = None;
        let _message_length = 0;
        
        for submessage in message.submessages() {
            match submessage {
                RtpsSubmessage::AckNack(ack_nack) => todo!(),
                RtpsSubmessage::Data(data) => {
                    let writer_guid = GUID::new(source_guid_prefix, data.writer_id());
                    let _reader_guid = GUID::new(dest_guid_prefix, data.reader_id());
                    // Find the reader with data.reader_id() and send this message to it. Messages for the discovery
                    // are going to a stateless reader, all others should be using a stateful reader which keeps
                    // a list of matched writers
                    for reader in &self.stateful_reader_list {
                        println!("Will do something with data message");
                        if reader.matched_writer_lookup(&writer_guid).is_some() {
                            // Send message to this reader/writer proxy
                            todo!();
                            break;
                        }
                    }
                },
                RtpsSubmessage::Gap(gap) => todo!(),
                RtpsSubmessage::Heartbeat(heartbeat) => todo!(),
                RtpsSubmessage::InfoTs(info_ts) => timestamp = info_ts.time(),
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

        let receiver = RtpsMessageReceiver::new(guid_prefix, &[&reader1]);

        let info_ts = InfoTs::new(Some(Time::new(100,100)), Endianness::LittleEndian);
        let data = Data::new(Endianness::LittleEndian,
            EntityId::new([1,2,3], EntityKind::UserDefinedReaderWithKey),
            EntityId::new([1,2,3], EntityKind::UserDefinedWriterWithKey),
            1, None, Payload::Data(vec![1,2,3,4]));
        let message = RtpsMessage::new([2;12],vec![RtpsSubmessage::InfoTs(info_ts), RtpsSubmessage::Data(data)]);

        receiver.receive(message);
    }
}