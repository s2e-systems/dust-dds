use crate::types::{Locator, LocatorList, ProtocolVersion, VendorId, ReliabilityKind, TopicKind, GUID, GuidPrefix, Duration, Time};
use crate::types::{ENTITYID_SPDP_BUILT_IN_PARTICIPANT_READER, DURATION_ZERO};
use crate::entity::Entity;
use crate::endpoint::{Endpoint};
use crate::reader::{Reader};
use crate::transport::Transport;
use crate::{Udpv4Locator};
use crate::parser::{RtpsMessage, parse_rtps_message, SubMessageType, InfoTs, InfoSrc, Data, Payload};

struct Participant{
    default_unicast_locator_list: LocatorList,
    default_multicast_locator_list: LocatorList,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    socket: Transport,
    spdp_builtin_participant_reader: Reader,
    // SPDPbuiltinParticipantWriter,
}

impl Participant{

    fn new(default_unicast_locator_list: LocatorList, default_multicast_locator_list: LocatorList, protocol_version: ProtocolVersion, vendor_id: VendorId,) -> Self {
        let guid_prefix = [5,6,7,8,9,5,1,2,3,4,10,11];

        let socket = Transport::new(Udpv4Locator::new_udpv4(&[127,0,0,1], &7400), Some(Udpv4Locator::new_udpv4(&[239,255,0,1], &7400))).unwrap();

        let endpoint = Endpoint {
            entity: Entity{guid: GUID::new(guid_prefix, ENTITYID_SPDP_BUILT_IN_PARTICIPANT_READER)},
            topic_kind: TopicKind::WithKey,
            reliability_level: ReliabilityKind::BestEffort,
            unicast_locator_list: default_unicast_locator_list.clone(),
            multicast_locator_list: default_multicast_locator_list.clone(),
        };

        let heartbeat_response_delay = DURATION_ZERO;
        let heartbeat_suppression_duration = DURATION_ZERO;
        let expects_inline_qos = false;
  
        let spdp_builtin_participant_reader = Reader::new(endpoint, heartbeat_response_delay, heartbeat_suppression_duration, expects_inline_qos);

        Participant{
            default_unicast_locator_list,
            default_multicast_locator_list,
            protocol_version,
            vendor_id,
            socket,
            spdp_builtin_participant_reader,
        }
    }

    fn receive_data(&mut self) {
        let received_data = self.socket.read().unwrap_or(&[]);
        println!("Data: {:?}", received_data);

        let rtps_message = parse_rtps_message(received_data);
        println!("RTPS message: {:?}", rtps_message);

        match rtps_message {
            Ok(message) => self.process_message(message),
            _ => (),
        }

        // TODO: Check if there are changes between participant proxy list and spdp_builtin_participant_reader history cache
        

    }

    pub fn process_message(&mut self, message: RtpsMessage) {
        let (mut source_guid_prefix, mut source_vendor_id, mut source_protocol_version, mut submessages) = message.take(); 
        let mut message_timestamp : Option<Time> = None;

        while let Some(submessage) = submessages.pop_front() {
            match submessage {
                SubMessageType::InfoTsSubmessage(info_ts) => Self::process_infots(info_ts, &mut message_timestamp),
                SubMessageType::DataSubmessage(data) => self.process_data(data, &source_guid_prefix),
                SubMessageType::InfoSrcSubmessage(info_src) => Self::process_infosrc(info_src, &mut source_protocol_version, &mut source_vendor_id, &mut source_guid_prefix),
                _ => println!("Unimplemented message type"),
            };  
        }
    }

    fn process_infots(info_ts: InfoTs, time: &mut Option<Time>) {
        *time = info_ts.take();
    }

    fn process_infosrc(info_src: InfoSrc, protocol_version: &mut ProtocolVersion, vendor_id: &mut VendorId, guid_prefix: &mut GuidPrefix) {
        let (new_protocol_version, new_vendor_id, new_guid_prefix)=info_src.take();
        *protocol_version = new_protocol_version;
        *vendor_id = new_vendor_id;
        *guid_prefix = new_guid_prefix;
    }

    fn process_data(&mut self, data_submessage: Data, source_guid_prefix: &GuidPrefix) {
        let (reader_id, writer_id, writer_sn, inline_qos, serialized_payload) = data_submessage.take();
        let writer_guid = GUID::new(*source_guid_prefix, writer_id);

        match writer_id {
            ENTITYID_SPDP_BUILT_IN_PARTICIPANT_WRITER => self.spdp_builtin_participant_reader.read_data(writer_guid, writer_sn, inline_qos, serialized_payload),
            
            _ => println!("Unknown data destination"),
        };
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::net::SocketAddr;
    use crate::cache::HistoryCache;

    #[test]
    fn test_participant() {
        let addr = [127,0,0,1];
        let multicast_group = [239,255,0,1];
        let port = 7400;
        let sender = std::net::UdpSocket::bind(SocketAddr::from((addr, 0))).unwrap();

        let vendor_id = [99,99];
        let protocol_version = ProtocolVersion{major:2, minor:4};
        let mut participant = Participant::new(vec![], vec![], protocol_version, vendor_id);

        let data = [0x52, 0x54, 0x50, 0x53,
        0x02, 0x01, 0x01, 0x02,
        0x7f, 0x20, 0xf7, 0xd7,
        0x00, 0x00, 0x01, 0xbb,
        0x00, 0x00, 0x00, 0x01,
        0x09, 0x01, 0x08, 0x00,
        0x9e, 0x81, 0xbc, 0x5d,
        0x97, 0xde, 0x48, 0x26,
        0x15, 0x07, 0x1c, 0x01,
        0x00, 0x00, 0x10, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x00, 0xc2,
        0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00,
        0x70, 0x00, 0x10, 0x00,
        0x7f, 0x20, 0xf7, 0xd7,
        0x00, 0x00, 0x01, 0xbb,
        0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x01, 0xc1,
        0x01, 0x00, 0x00, 0x00,
        0x00, 0x03, 0x00, 0x00,
        0x15, 0x00, 0x04, 0x00,
        0x02, 0x01, 0x00, 0x00,
        0x16, 0x00, 0x04, 0x00,
        0x01, 0x02, 0x00, 0x00,
        0x31, 0x00, 0x18, 0x00,
        0x01, 0x00, 0x00, 0x00,
        0xf3, 0x1c, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0xc0, 0xa8, 0x02, 0x04,
        0x32, 0x00, 0x18, 0x00,
        0x01, 0x00, 0x00, 0x00,
        0xf2, 0x1c, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0xc0, 0xa8, 0x02, 0x04,
        0x02, 0x00, 0x08, 0x00,
        0x0b, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x50, 0x00, 0x10, 0x00,
        0x7f, 0x20, 0xf7, 0xd7,
        0x00, 0x00, 0x01, 0xbb,
        0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x01, 0xc1,
        0x58, 0x00, 0x04, 0x00,
        0x15, 0x04, 0x00, 0x00,
        0x00, 0x80, 0x04, 0x00,
        0x15, 0x00, 0x00, 0x00,
        0x07, 0x80, 0x5c, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x2f, 0x00, 0x00, 0x00,
        0x05, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x50, 0x00, 0x00, 0x00,
        0x42, 0x00, 0x00, 0x00,
        0x44, 0x45, 0x53, 0x4b,
        0x54, 0x4f, 0x50, 0x2d,
        0x4f, 0x52, 0x46, 0x44,
        0x4f, 0x53, 0x35, 0x2f,
        0x36, 0x2e, 0x31, 0x30,
        0x2e, 0x32, 0x2f, 0x63,
        0x63, 0x36, 0x66, 0x62,
        0x39, 0x61, 0x62, 0x33,
        0x36, 0x2f, 0x39, 0x30,
        0x37, 0x65, 0x66, 0x66,
        0x30, 0x32, 0x65, 0x33,
        0x2f, 0x22, 0x78, 0x38,
        0x36, 0x5f, 0x36, 0x34,
        0x2e, 0x77, 0x69, 0x6e,
        0x2d, 0x76, 0x73, 0x32,
        0x30, 0x31, 0x35, 0x22,
        0x2f, 0x00, 0x00, 0x00,
        0x25, 0x80, 0x0c, 0x00,
        0xd7, 0xf7, 0x20, 0x7f,
        0xbb, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00];
        sender.send_to(&data, SocketAddr::from((multicast_group, port))).unwrap();

        assert_eq!(participant.spdp_builtin_participant_reader.reader_cache.get_changes().len(),0);

        participant.receive_data();
        
        assert_eq!(participant.spdp_builtin_participant_reader.reader_cache.get_changes().len(),1);
    }
}
