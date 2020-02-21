use std::collections::HashSet;

use crate::cache::HistoryCache;
use crate::endpoint::Endpoint;
use crate::entity::Entity;
use crate::parser::{
    parse_rtps_message, Data, InfoSrc, InfoTs, Payload, RtpsMessage, SubMessageType,
};
use crate::participant_proxy::ParticipantProxy;
use crate::proxy::ReaderProxy;
use crate::reader::Reader;
use crate::transport::Transport;
use crate::types::{
    BuiltInEndPoints, Duration, GuidPrefix, InlineQosParameterList, Locator, LocatorList,
    ProtocolVersion, ReliabilityKind, SequenceNumber, Time, TopicKind, VendorId, GUID,
};
use crate::types::{
    DURATION_ZERO, ENTITYID_PARTICIPANT, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
    ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR, ENTITYID_UNKNOWN,
};
use crate::writer::StatefulWriter;
use crate::Udpv4Locator;

struct Participant {
    entity: Entity,
    default_unicast_locator_list: LocatorList,
    default_multicast_locator_list: LocatorList,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    socket: Transport,
    spdp_builtin_participant_reader: Reader,
    // spdp_builtin_participant_writer: StatelessWriter,
    sedp_builtin_publications_writer: StatefulWriter,
    participant_proxy_list: HashSet<ParticipantProxy>,
}

impl Participant {
    fn new(
        default_unicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];
        let participant_guid = GUID::new_participant_guid(guid_prefix);

        let socket = Transport::new(
            Udpv4Locator::new_udpv4(&[127, 0, 0, 1], &7400),
            Some(Udpv4Locator::new_udpv4(&[239, 255, 0, 1], &7400)),
        )
        .unwrap();

        let endpoint = Endpoint::new(
            Entity {
                guid: GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR),
            },
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            default_unicast_locator_list.clone(),
            default_multicast_locator_list.clone(),
        );

        let heartbeat_response_delay = DURATION_ZERO;
        let heartbeat_suppression_duration = DURATION_ZERO;
        let expects_inline_qos = false;

        let spdp_builtin_participant_reader = Reader::new(
            endpoint,
            heartbeat_response_delay,
            heartbeat_suppression_duration,
            expects_inline_qos,
        );

        let writer_endpoint = Endpoint::new(
            Entity {
                guid: GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
            },
            TopicKind::WithKey,
            ReliabilityKind::Reliable,
            default_unicast_locator_list.clone(),
            default_multicast_locator_list.clone(),
        );
        let sedp_builtin_publications_writer = StatefulWriter::new(
            writer_endpoint,
            true,          /*push_mode*/
            DURATION_ZERO, /*heartbeat_period*/
            DURATION_ZERO, /*nack_response_delay*/
            DURATION_ZERO, /*nack_suppression_duration*/
        );
        Participant {
            entity: Entity {
                guid: participant_guid,
            },
            default_unicast_locator_list,
            default_multicast_locator_list,
            protocol_version,
            vendor_id,
            socket,
            spdp_builtin_participant_reader,
            sedp_builtin_publications_writer,
            // sedp_builtin_publications_reader,
            participant_proxy_list: HashSet::new(),
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
        let (
            mut source_guid_prefix,
            mut source_vendor_id,
            mut source_protocol_version,
            mut submessages,
        ) = message.take();
        let mut message_timestamp: Option<Time> = None;

        while let Some(submessage) = submessages.pop_front() {
            match submessage {
                SubMessageType::InfoTsSubmessage(info_ts) => {
                    Self::process_infots(info_ts, &mut message_timestamp)
                }
                SubMessageType::DataSubmessage(data) => {
                    self.process_data(data, &source_guid_prefix)
                }
                SubMessageType::InfoSrcSubmessage(info_src) => Self::process_infosrc(
                    info_src,
                    &mut source_protocol_version,
                    &mut source_vendor_id,
                    &mut source_guid_prefix,
                ),
                _ => println!("Unimplemented message type"),
            };
        }
    }

    fn process_infots(info_ts: InfoTs, time: &mut Option<Time>) {
        *time = info_ts.take();
    }

    fn process_infosrc(
        info_src: InfoSrc,
        protocol_version: &mut ProtocolVersion,
        vendor_id: &mut VendorId,
        guid_prefix: &mut GuidPrefix,
    ) {
        let (new_protocol_version, new_vendor_id, new_guid_prefix) = info_src.take();
        *protocol_version = new_protocol_version;
        *vendor_id = new_vendor_id;
        *guid_prefix = new_guid_prefix;
    }

    fn process_data(&mut self, data_submessage: Data, source_guid_prefix: &GuidPrefix) {
        let (reader_id, writer_id, writer_sn, inline_qos, serialized_payload) =
            data_submessage.take();
        let writer_guid = GUID::new(*source_guid_prefix, writer_id);

        match writer_id {
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER => {
                self.process_spdp(writer_guid, writer_sn, inline_qos, serialized_payload)
            }
            _ => println!("Unknown data destination"),
        };
    }

    fn process_spdp(
        &mut self,
        writer_guid: GUID,
        sequence_number: SequenceNumber,
        inline_qos: Option<InlineQosParameterList>,
        serialized_payload: Payload,
    ) {
        self.spdp_builtin_participant_reader.read_data(
            writer_guid,
            sequence_number,
            inline_qos,
            serialized_payload,
        );
        let mut participant_proxy_list = HashSet::new();
        for change in self
            .spdp_builtin_participant_reader
            .reader_cache
            .get_changes()
            .iter()
        {
            let participant_proxy = ParticipantProxy::new(change.data().unwrap()).unwrap();
            
            participant_proxy_list
                .insert(participant_proxy);
        }
        for participant_proxy in participant_proxy_list.iter() {
            self.add_reader_proxy_for_sedp(&participant_proxy);
        }

        self.participant_proxy_list = participant_proxy_list;
    }

    fn add_reader_proxy_for_sedp(&mut self, participant_proxy: &ParticipantProxy) {
        if participant_proxy
            .available_builtin_endpoints()
            .has(BuiltInEndPoints::PublicationsDetector)
        {
            let guid = GUID::new(
                *participant_proxy.guid_prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let reader_proxy = ReaderProxy::new(
                guid,
                ENTITYID_UNKNOWN,
                participant_proxy.metatraffic_unicast_locator_list.clone(),
                participant_proxy.metatraffic_multicast_locator_list.clone(),
                participant_proxy.expects_inline_qos,
                true, /*is_active*/
            );
            self.sedp_builtin_publications_writer.matched_reader_add(reader_proxy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::HistoryCache;
    use std::net::SocketAddr;

    #[test]
    fn test_participant() {
        let addr = [127, 0, 0, 1];
        let multicast_group = [239, 255, 0, 1];
        let port = 7400;
        let sender = std::net::UdpSocket::bind(SocketAddr::from((addr, 0))).unwrap();

        let vendor_id = [99, 99];
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let mut participant = Participant::new(vec![], vec![], protocol_version, vendor_id);

        let mut data = [
            0x52, 0x54, 0x50, 0x53, 0x02, 0x01, 0x01, 0x02, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00,
            0x01, 0xbb, 0x00, 0x00, 0x00, 0x01, 0x09, 0x01, 0x08, 0x00, 0x9e, 0x81, 0xbc, 0x5d,
            0x97, 0xde, 0x48, 0x26, 0x15, 0x07, 0x1c, 0x01, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x70, 0x00, 0x10, 0x00, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x01, 0xc1, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
            0x15, 0x00, 0x04, 0x00, 0x02, 0x01, 0x00, 0x00, 0x16, 0x00, 0x04, 0x00, 0x01, 0x02,
            0x00, 0x00, 0x31, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf3, 0x1c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8,
            0x02, 0x04, 0x32, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf2, 0x1c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8,
            0x02, 0x04, 0x02, 0x00, 0x08, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x50, 0x00, 0x10, 0x00, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x01, 0xc1, 0x58, 0x00, 0x04, 0x00, 0x15, 0x04, 0x00, 0x00,
            0x00, 0x80, 0x04, 0x00, 0x15, 0x00, 0x00, 0x00, 0x07, 0x80, 0x5c, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x2f, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x50, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x44, 0x45, 0x53, 0x4b, 0x54, 0x4f,
            0x50, 0x2d, 0x4f, 0x52, 0x46, 0x44, 0x4f, 0x53, 0x35, 0x2f, 0x36, 0x2e, 0x31, 0x30,
            0x2e, 0x32, 0x2f, 0x63, 0x63, 0x36, 0x66, 0x62, 0x39, 0x61, 0x62, 0x33, 0x36, 0x2f,
            0x39, 0x30, 0x37, 0x65, 0x66, 0x66, 0x30, 0x32, 0x65, 0x33, 0x2f, 0x22, 0x78, 0x38,
            0x36, 0x5f, 0x36, 0x34, 0x2e, 0x77, 0x69, 0x6e, 0x2d, 0x76, 0x73, 0x32, 0x30, 0x31,
            0x35, 0x22, 0x2f, 0x00, 0x00, 0x00, 0x25, 0x80, 0x0c, 0x00, 0xd7, 0xf7, 0x20, 0x7f,
            0xbb, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        ];
        sender
            .send_to(&data, SocketAddr::from((multicast_group, port)))
            .unwrap();

        assert_eq!(
            participant
                .spdp_builtin_participant_reader
                .reader_cache
                .get_changes()
                .len(),
            0
        );

        assert_eq!(participant.participant_proxy_list.len(), 0);

        participant.receive_data();

        assert_eq!(
            participant
                .spdp_builtin_participant_reader
                .reader_cache
                .get_changes()
                .len(),
            1
        );

        assert_eq!(participant.participant_proxy_list.len(), 1);

        // Change the source GUID prefix:
        // This should result in no new participant proxy
        data[9] = 99;
        sender
            .send_to(&data, SocketAddr::from((multicast_group, port)))
            .unwrap();

        participant.receive_data();

        assert_eq!(
            participant
                .spdp_builtin_participant_reader
                .reader_cache
                .get_changes()
                .len(),
            2
        );

        assert_eq!(participant.participant_proxy_list.len(), 1);

        // Change the InstanceHandle (GUID prefix of the KeyHash):
        data[61] = 99;
        sender
            .send_to(&data, SocketAddr::from((multicast_group, port)))
            .unwrap();

        participant.receive_data();

        assert_eq!(
            participant
                .spdp_builtin_participant_reader
                .reader_cache
                .get_changes()
                .len(),
            3
        );

        assert_eq!(participant.participant_proxy_list.len(), 1);

        // Change the InstanceHandle (GUID prefix of the KeyHash):
        data[173] = 99;
        sender
            .send_to(&data, SocketAddr::from((multicast_group, port)))
            .unwrap();

        participant.receive_data();

        assert_eq!(
            participant
                .spdp_builtin_participant_reader
                .reader_cache
                .get_changes()
                .len(),
            3
        );

        assert_eq!(participant.participant_proxy_list.len(), 2);
    }
}
