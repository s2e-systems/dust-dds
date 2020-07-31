use crate::stateless_writer::StatelessWriter;
use crate::types::{GUID, Locator, ProtocolVersion, VendorId, TopicKind, ChangeKind};
use crate::types::constants::{ENTITYID_PARTICIPANT, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER};

pub struct Participant {
    guid: GUID,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    // socket: Transport,
    // spdp_builtin_participant_reader: StatelessReader,
    spdp_builtin_participant_writer: StatelessWriter,
    // sedp_builtin_publications_reader: StatefulReader,
    // sedp_builtin_publications_writer: StatefulWriter,
    // sedp_builtin_subscriptions_reader: StatefulReader,
    // sedp_builtin_subscriptions_writer: StatefulWriter,
    // sedp_builtin_topics_reader: StatefulReader,
    // sedp_builtin_topics_writer: StatefulWriter,
    // participant_proxy_list: HashSet<ParticipantProxy>,
}

impl Participant {
    fn new(
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {

        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];

        let spdp_builtin_participant_writer = StatelessWriter::new(
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
            TopicKind::WithKey);

        // TODO: Add the SPDP discovery locator
        // spdp_builtin_participant_writer.reader_locator_add(Locator::new())

        let participant = Self {
            guid: GUID::new(guid_prefix,ENTITYID_PARTICIPANT ),
            default_unicast_locator_list,
            default_multicast_locator_list,
            protocol_version,
            vendor_id,
            spdp_builtin_participant_writer,
        };

        // TODO: Get the participant which is described here 
        // spdp_builtin_participant_writer.new_change(ChangeKind::Alive,Some(vec![]) , None, guid_prefix/*handle*/);
        
        participant
        

    //     let socket = Transport::new(
    //         Udpv4Locator::new_udpv4(&[127, 0, 0, 1], &7400),
    //         Some(Udpv4Locator::new_udpv4(&[239, 255, 0, 1], &7400)),
    //     )
    //     .unwrap();

    //     let heartbeat_response_delay = DURATION_ZERO;
    //     let heartbeat_suppression_duration = DURATION_ZERO;
    //     let expects_inline_qos = false;

    //     let spdp_builtin_participant_reader = StatelessReader::new(
    //         GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR),
    //         TopicKind::WithKey,
    //         ReliabilityKind::BestEffort,
    //         default_unicast_locator_list.clone(),
    //         default_multicast_locator_list.clone(),
    //         heartbeat_response_delay.clone(),
    //         heartbeat_suppression_duration.clone(),
    //         expects_inline_qos,
    //     );

    //     let spdp_builtin_participant_writer = StatelessWriter::new(
    //         GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
    //         TopicKind::WithKey,
    //         ReliabilityKind::BestEffort,
    //         default_unicast_locator_list.clone(),
    //         default_multicast_locator_list.clone(),
    //         true,          /*push_mode*/
    //         DURATION_ZERO, /*heartbeat_period*/
    //         DURATION_ZERO, /*nack_response_delay*/
    //         DURATION_ZERO, /*nack_suppression_duration*/
    //     );

    //     let sedp_builtin_publications_reader = StatefulReader::new(
    //         GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
    //         TopicKind::WithKey,
    //         ReliabilityKind::Reliable,
    //         default_unicast_locator_list.clone(),
    //         default_multicast_locator_list.clone(),
    //         heartbeat_response_delay.clone(),
    //         heartbeat_suppression_duration.clone(),
    //         expects_inline_qos,
    //     );

    //     let sedp_builtin_publications_writer = StatefulWriter::new(
    //         GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
    //         TopicKind::WithKey,
    //         ReliabilityKind::Reliable,
    //         default_unicast_locator_list.clone(),
    //         default_multicast_locator_list.clone(),
    //         true,          /*push_mode*/
    //         DURATION_ZERO, /*heartbeat_period*/
    //         DURATION_ZERO, /*nack_response_delay*/
    //         DURATION_ZERO, /*nack_suppression_duration*/
    //     );

    //     let sedp_builtin_subscriptions_reader = StatefulReader::new(
    //         GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
    //         TopicKind::WithKey,
    //         ReliabilityKind::Reliable,
    //         default_unicast_locator_list.clone(),
    //         default_multicast_locator_list.clone(),
    //         heartbeat_response_delay.clone(),
    //         heartbeat_suppression_duration.clone(),
    //         expects_inline_qos,
    //     );

    //     let sedp_builtin_subscriptions_writer = StatefulWriter::new(
    //         GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
    //         TopicKind::WithKey,
    //         ReliabilityKind::Reliable,
    //         default_unicast_locator_list.clone(),
    //         default_multicast_locator_list.clone(),
    //         true,          /*push_mode*/
    //         DURATION_ZERO, /*heartbeat_period*/
    //         DURATION_ZERO, /*nack_response_delay*/
    //         DURATION_ZERO, /*nack_suppression_duration*/
    //     );

    //     let sedp_builtin_topics_reader = StatefulReader::new(
    //         GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
    //         TopicKind::WithKey,
    //         ReliabilityKind::Reliable,
    //         default_unicast_locator_list.clone(),
    //         default_multicast_locator_list.clone(),
    //         heartbeat_response_delay.clone(),
    //         heartbeat_suppression_duration.clone(),
    //         expects_inline_qos,
    //     );

    //     let sedp_builtin_topics_writer = StatefulWriter::new(
    //         GUID::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
    //         TopicKind::WithKey,
    //         ReliabilityKind::Reliable,
    //         default_unicast_locator_list.clone(),
    //         default_multicast_locator_list.clone(),
    //         true,          /*push_mode*/
    //         DURATION_ZERO, /*heartbeat_period*/
    //         DURATION_ZERO, /*nack_response_delay*/
    //         DURATION_ZERO, /*nack_suppression_duration*/
    //     );

    //     let mut new_participant = Participant {
    //         entity: Entity {
    //             guid: GUID::new(guid_prefix, ENTITYID_PARTICIPANT),
    //         },
    //         default_unicast_locator_list,
    //         default_multicast_locator_list,
    //         protocol_version,
    //         vendor_id,
    //         socket,
    //         spdp_builtin_participant_reader,
    //         spdp_builtin_participant_writer,
    //         sedp_builtin_publications_reader,
    //         sedp_builtin_publications_writer,
    //         sedp_builtin_subscriptions_reader,
    //         sedp_builtin_subscriptions_writer,
    //         sedp_builtin_topics_reader,
    //         sedp_builtin_topics_writer,
    //         participant_proxy_list: HashSet::new(),
    //     };

    //     new_participant.add_participant_to_spdp_writer();

    //     new_participant.spdp_builtin_participant_writer.reader_locator_add(Locator::new(0 /*UDP_V4_KIND*/,7400, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1]));

    //     new_participant
    }

    // fn add_participant_to_spdp_writer(&mut self) {
    //     let participant_data = cdr::serialize::<_,_,PlCdrLe>(self, Infinite).unwrap();
    //     let change = self.spdp_builtin_participant_writer.new_change(ChangeKind::Alive, Some(participant_data), None, [0;16]);
    //     self.spdp_builtin_participant_writer.history_cache().add_change(change);
    // }

    // fn receive_data(&mut self) {
    //     // let received_data = self.socket.read().unwrap_or(&[]);
    //     // println!("Data: {:?}", received_data);

    //     // let rtps_message = parse_rtps_message(received_data);
    //     // println!("RTPS message: {:?}", rtps_message);

    //     // TODO: Check if there are changes between participant proxy list and spdp_builtin_participant_reader history cache
    // }

    // fn send_data(&mut self) {
    //     let multicast_locator = Locator::new(0 /*UDP_V4_KIND*/,7400, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1]);
    //     let _spdp_data = self.spdp_builtin_participant_writer.get_data_to_send(multicast_locator); // Returns a vec of [Data(1) Data(2)]

    //     // let buf = serialize(spdp_data);
    //     // self.socket.write(buf, unicast_locator: Udpv4Locator)

    // }

    // fn process_spdp(
    //     &mut self,
    //     writer_guid: GUID,
    //     sequence_number: SequenceNumber,
    //     inline_qos: Option<InlineQosParameterList>,
    //     serialized_payload: Payload,
    // ) {
    //     self.spdp_builtin_participant_reader.read_data(
    //         writer_guid,
    //         sequence_number,
    //         inline_qos,
    //         serialized_payload,
    //     );
    //     let mut participant_proxy_list = HashSet::new();
    //     for change in self
    //         .spdp_builtin_participant_reader
    //         .reader_cache
    //         .get_changes()
    //         .iter()
    //     {
    //         let data = change.data().unwrap();
    //         let participant_proxy = ParticipantProxy::new_from_data(data).unwrap();

    //         participant_proxy_list.insert(participant_proxy);
    //     }
    //     for participant_proxy in participant_proxy_list.iter() {
    //         self.add_sedp_proxies(&participant_proxy);
    //     }

    //     self.participant_proxy_list = participant_proxy_list;
    // }

    // fn add_sedp_proxies(&mut self, participant_proxy: &ParticipantProxy) {

    //     // Publications
    //     if participant_proxy
    //         .available_builtin_endpoints()
    //         .has(BuiltInEndPoints::PublicationsDetector)
    //     {
    //         let proxy = ReaderProxy::new(
    //             GUID::new(
    //                 *participant_proxy.guid_prefix(),
    //                 ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    //             ),
    //             ENTITYID_UNKNOWN,
    //             participant_proxy.metatraffic_unicast_locator_list().clone(),
    //             participant_proxy.metatraffic_multicast_locator_list().clone(),
    //             participant_proxy.expects_inline_qos(),
    //             true, /*is_active*/
    //         );
    //         self.sedp_builtin_publications_writer
    //             .matched_reader_add(proxy);
    //     }

    //     if participant_proxy
    //         .available_builtin_endpoints()
    //         .has(BuiltInEndPoints::PublicationsAnnouncer)
    //     {
    //         let proxy = WriterProxy::new(
    //             GUID::new(
    //                 *participant_proxy.guid_prefix(),
    //                 ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    //             ),
    //             participant_proxy.metatraffic_unicast_locator_list().clone(),
    //             participant_proxy.metatraffic_multicast_locator_list().clone(),
    //             None,             /*data_max_size_serialized*/
    //             ENTITYID_UNKNOWN, /*remote_group_entity_id*/
    //         );
    //         self.sedp_builtin_publications_reader
    //             .matched_writer_add(proxy);
    //     }

        
    //     // Subscribtions

    //     if participant_proxy
    //         .available_builtin_endpoints()
    //         .has(BuiltInEndPoints::SubscriptionsDetector)
    //     {
    //         let proxy = ReaderProxy::new(
    //             GUID::new(
    //                 *participant_proxy.guid_prefix(),
    //                 ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    //             ),
    //             ENTITYID_UNKNOWN,
    //             participant_proxy.metatraffic_unicast_locator_list().clone(),
    //             participant_proxy.metatraffic_multicast_locator_list().clone(),
    //             participant_proxy.expects_inline_qos(),
    //             true, /*is_active*/
    //         );
    //         self.sedp_builtin_subscriptions_writer
    //             .matched_reader_add(proxy);
    //     }

        
    //     if participant_proxy
    //         .available_builtin_endpoints()
    //         .has(BuiltInEndPoints::SubscriptionsAnnouncer)
    //     {
    //         let proxy = WriterProxy::new(
    //             GUID::new(
    //                 *participant_proxy.guid_prefix(),
    //                 ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    //             ),
    //             participant_proxy.metatraffic_unicast_locator_list().clone(),
    //             participant_proxy.metatraffic_multicast_locator_list().clone(),
    //             None,             /*data_max_size_serialized*/
    //             ENTITYID_UNKNOWN, /*remote_group_entity_id*/
    //         );
    //         self.sedp_builtin_subscriptions_reader
    //             .matched_writer_add(proxy);
    //     }

    //     // Topics

    //     if participant_proxy
    //         .available_builtin_endpoints()
    //         .has(BuiltInEndPoints::TopicsDetector)
    //     {
    //         let proxy = ReaderProxy::new(
    //             GUID::new(
    //                 *participant_proxy.guid_prefix(),
    //                 ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
    //             ),
    //             ENTITYID_UNKNOWN,
    //             participant_proxy.metatraffic_unicast_locator_list().clone(),
    //             participant_proxy.metatraffic_multicast_locator_list().clone(),
    //             participant_proxy.expects_inline_qos(),
    //             true, /*is_active*/
    //         );
    //         self.sedp_builtin_topics_writer
    //             .matched_reader_add(proxy);
    //     }
        
    //     if participant_proxy
    //         .available_builtin_endpoints()
    //         .has(BuiltInEndPoints::TopicsAnnouncer)
    //     {
    //         let proxy = WriterProxy::new(
    //             GUID::new(
    //                 *participant_proxy.guid_prefix(),
    //                 ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    //             ),
    //             participant_proxy.metatraffic_unicast_locator_list().clone(),
    //             participant_proxy.metatraffic_multicast_locator_list().clone(),
    //             None,             /*data_max_size_serialized*/
    //             ENTITYID_UNKNOWN, /*remote_group_entity_id*/
    //         );
    //         self.sedp_builtin_topics_reader
    //             .matched_writer_add(proxy);
    //     }
    // }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::cache::HistoryCache;
    // use std::net::SocketAddr;

    // #[test]
    // fn test_participant() {
    //     let addr = [127, 0, 0, 1];
    //     let multicast_group = [239, 255, 0, 1];
    //     let port = 7400;
    //     let sender = std::net::UdpSocket::bind(SocketAddr::from((addr, 0))).unwrap();

    //     let vendor_id = [99, 99];
    //     let protocol_version = ProtocolVersion { major: 2, minor: 4 };
    //     let mut participant = Participant::new(vec![], vec![], protocol_version, vendor_id);

    //     let data = [
    //         0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
    //         0x02, 0x01, 0x01, 0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
    //         0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
    //         0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
    //         0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
    //         0x09, 0x01, 0x08, 0x00, //020 submessageId: SubmessageKind => INFO_TS | flags: SubmessageFlag[8] => Endianess=little | submessageLength: ushort => 8
    //         0x9e, 0x81, 0xbc, 0x5d, //024  [InfoTimestamp Submessage]
    //         0x97, 0xde, 0x48, 0x26, //028  [InfoTimestamp Submessage]
    //         0x15, 0x07, 0x1c, 0x01, //032 submessageId: SubmessageKind => DATA | flags: SubmessageFlag[8] => N=0|K=0|D=1|Q=1|E=1 Endianess=little && InlineQosFlag && serializedPayload contains data | submessageLength (octetsToNextHeader): ushort => 284
    //         0x00, 0x00, 0x10, 0x00, //036  [Data Submessage] Flags: extraFlags | octetsToInlineQos: ushort => 16
    //         0x00, 0x00, 0x00, 0x00, //040  [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
    //         0x00, 0x01, 0x00, 0xc2, //044  [Data Submessage] EntityId writerId => ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR ([0, 0x01, 0x00], ENTITY_KIND_BUILT_IN_READER_WITH_KEY)
    //         0x00, 0x00, 0x00, 0x00, //048  [Data Submessage] SequenceNumber writerSN 
    //         0x01, 0x00, 0x00, 0x00, //052  [Data Submessage] SequenceNumber writerSN => 1
    //         0x70, 0x00, 0x10, 0x00, //056  [Data Submessage: inLineQos] parameterId_1: short => PID_KEY_HASH | length: short => 16
    //         0x7f, 0x20, 0xf7, 0xd7, //060  [Data Submessage: inLineQos: KEY_HASH] 
    //         0x00, 0x00, 0x01, 0xbb, //064  [Data Submessage: inLineQos: KEY_HASH] 
    //         0x00, 0x00, 0x00, 0x01, //068  [Data Submessage: inLineQos: KEY_HASH]  
    //         0x00, 0x00, 0x01, 0xc1, //072  [Data Submessage: inLineQos: KEY_HASH]  
    //         0x01, 0x00, 0x00, 0x00, //076  [Data Submessage]  parameterId_1: short => PID_SENTINEL | 0
    //         0x00, 0x03, 0x00, 0x00, //080  [Data Submessage: SerializedPayload]   representation_identifier: octet[2] => PL_CDR_LE | representation_options: octet[2] => none
    //         0x15, 0x00, 0x04, 0x00, //084  [Data Submessage: SerializedPayload]   parameterId_1: short => PID_PROTOCOL_VERSION | length: short => 4
    //         0x02, 0x01, 0x00, 0x00, //088  [Data Submessage: SerializedPayload: PID_PROTOCOL_VERSION]  major: octet => 2 | minor: octet =>1 | padding 
    //         0x16, 0x00, 0x04, 0x00, //092  [Data Submessage: SerializedPayload]  parameterId_1: short => PID_VENDORID  | length: short => 4
    //         0x01, 0x02, 0x00, 0x00, //096  [Data Submessage: SerializedPayload: PID_VENDORID] vendorId: octet[2] => 12
    //         0x31, 0x00, 0x18, 0x00, //100  [Data Submessage: SerializedPayload]  parameterId_1: short =>  PID_DEFAULT_UNICAST_LOCATOR | length: short => 24
    //         0x01, 0x00, 0x00, 0x00, //104  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xf3, 0x1c, 0x00, 0x00, //108  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, //112  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, //116  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]  
    //         0x00, 0x00, 0x00, 0x00, //120  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xc0, 0xa8, 0x02, 0x04, //124  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]
    //         0x32, 0x00, 0x18, 0x00, //128  [Data Submessage: SerializedPayload] parameterId_1: short => PID_METATRAFFIC_UNICAST_LOCATOR | length: short => 24
    //         0x01, 0x00, 0x00, 0x00, //132  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xf2, 0x1c, 0x00, 0x00, //136  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, //140  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, //144  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]  
    //         0x00, 0x00, 0x00, 0x00, //148  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xc0, 0xa8, 0x02, 0x04, //152  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x02, 0x00, 0x08, 0x00, //156  [Data Submessage: SerializedPayload] parameterId_1: short => PID_PARTICIPANT_LEASE_DURATION | length: short => 8
    //         0x0b, 0x00, 0x00, 0x00, //160  [Data Submessage: SerializedPayload: PID_PARTICIPANT_LEASE_DURATION] seconds: long => 11 
    //         0x00, 0x00, 0x00, 0x00, //164  [Data Submessage: SerializedPayload: PID_PARTICIPANT_LEASE_DURATION] fraction: ulong => 0    
    //         0x50, 0x00, 0x10, 0x00, //168  [Data Submessage: SerializedPayload] parameterId_1: short => PID_PARTICIPANT_GUID | length: short => 16
    //         0x7f, 0x20, 0xf7, 0xd7, //172  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID] 
    //         0x00, 0x00, 0x01, 0xbb, //176  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x00, 0x00, 0x00, 0x01, //180  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x00, 0x00, 0x01, 0xc1, //184  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x58, 0x00, 0x04, 0x00, //188  [Data Submessage: SerializedPayload] parameterId_1: short => PID_BUILTIN_ENDPOINT_SET | length: short => 4
    //         0x15, 0x04, 0x00, 0x00, //192  [Data Submessage: SerializedPayload: PID_BUILTIN_ENDPOINT_SET] BuiltinEndpointSet: bitmask => (0100 0001 0101‬) PARTICIPANT_ANNOUNCER && PUBLICATIONS_ANNOUNCER && SUBSCRIPTIONS_ANNOUNCER && PARTICIPANT_MESSAGE_DATA_WRITER
    //         0x00, 0x80, 0x04, 0x00, //196  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8000) | length: short => 4   
    //         0x15, 0x00, 0x00, 0x00, //200  [Data Submessage: SerializedPayload: Vendor-specific 0x0]  
    //         0x07, 0x80, 0x5c, 0x00, //204  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8007) | length: short => 92     
    //         0x00, 0x00, 0x00, 0x00, //208  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2f, 0x00, 0x00, 0x00, //212  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x05, 0x00, 0x00, 0x00, //216  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x00, 0x00, 0x00, 0x00, //220  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x50, 0x00, 0x00, 0x00, //224  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x42, 0x00, 0x00, 0x00, //228  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
    //         0x44, 0x45, 0x53, 0x4b, //232  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x54, 0x4f, 0x50, 0x2d, //236  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x4f, 0x52, 0x46, 0x44, //240  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x4f, 0x53, 0x35, 0x2f, //244  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x36, 0x2e, 0x31, 0x30, //248  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2e, 0x32, 0x2f, 0x63, //252  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
    //         0x63, 0x36, 0x66, 0x62, //256  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x39, 0x61, 0x62, 0x33, //260  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x36, 0x2f, 0x39, 0x30, //264  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x37, 0x65, 0x66, 0x66, //268  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x30, 0x32, 0x65, 0x33, //272  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2f, 0x22, 0x78, 0x38, //276  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x36, 0x5f, 0x36, 0x34, //280  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2e, 0x77, 0x69, 0x6e, //284  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
    //         0x2d, 0x76, 0x73, 0x32, //288  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x30, 0x31, 0x35, 0x22, //292  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2f, 0x00, 0x00, 0x00, //296  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x25, 0x80, 0x0c, 0x00, //300  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8025) | length: short => 12       
    //         0xd7, 0xf7, 0x20, 0x7f, //304  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]   
    //         0xbb, 0x01, 0x00, 0x00, //308  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]   
    //         0x01, 0x00, 0x00, 0x00, //312  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]  
    //         0x01, 0x00, 0x00, 0x00, //316  [Data Submessage: SerializedPayload] parameterId_1: short => PID_SENTINEL |  length: short => 0
    //     ];
    //     sender
    //         .send_to(&data, SocketAddr::from((multicast_group, port)))
    //         .unwrap();

    //     assert_eq!(
    //         participant
    //             .spdp_builtin_participant_reader
    //             .reader_cache
    //             .get_changes()
    //             .len(),
    //         0
    //     );

    //     assert_eq!(participant.participant_proxy_list.len(), 0);

    //     participant.receive_data();

    //     assert_eq!(
    //         participant
    //             .spdp_builtin_participant_reader
    //             .reader_cache
    //             .get_changes()
    //             .len(),
    //         1
    //     );

    //     assert_eq!(participant.participant_proxy_list.len(), 1);
    // }

    // #[test]
    // fn create_participant_proxy_data() {
    //     let vendor_id = [0x01, 0x42];
    //     let protocol_version = ProtocolVersion { major: 2, minor: 1 };
    //     let default_unicast_address = [0,0,0,0,0,0,0,0,0,0,0,0,192,168,2,4];
    //     let metatraffic_multicast_address = [0,0,0,0,0,0,0,0,0,0,0,0,192,168,2,4];
    //     let participant = Participant::new(vec![Locator::new(1,7411,default_unicast_address)], vec![Locator::new(1,7410,metatraffic_multicast_address)], protocol_version, vendor_id);
    //     let data = vec![
    //         0x00, 0x03, 0x00, 0x00, // [Data Submessage: SerializedPayload]   representation_identifier: octet[2] => PL_CDR_LE | representation_options: octet[2] => none
    //         0x15, 0x00, 0x04, 0x00, // [Data Submessage: SerializedPayload]   parameterId_1: short => PID_PROTOCOL_VERSION | length: short => 4
    //         0x02, 0x01, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_PROTOCOL_VERSION]  major: octet => 2 | minor: octet =>1 | padding 
    //         0x16, 0x00, 0x04, 0x00, // [Data Submessage: SerializedPayload]  parameterId_1: short => PID_VENDORID  | length: short => 4
    //         0x01, 0x42, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_VENDORID] vendorId: octet[2] => 12
    //         0x31, 0x00, 0x18, 0x00, // [Data Submessage: SerializedPayload]  parameterId_1: short =>  PID_DEFAULT_UNICAST_LOCATOR | length: short => 24
    //         0x01, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xf3, 0x1c, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]  
    //         0x00, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xc0, 0xa8, 0x02, 0x04, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]
    //         0x32, 0x00, 0x18, 0x00, // [Data Submessage: SerializedPayload] parameterId_1: short => PID_METATRAFFIC_UNICAST_LOCATOR | length: short => 24
    //         0x01, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xf2, 0x1c, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]  
    //         0x00, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xc0, 0xa8, 0x02, 0x04, // [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x02, 0x00, 0x08, 0x00, // [Data Submessage: SerializedPayload] parameterId_1: short => PID_PARTICIPANT_LEASE_DURATION | length: short => 8
    //         0x0b, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_PARTICIPANT_LEASE_DURATION] seconds: long => 11 
    //         0x00, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_PARTICIPANT_LEASE_DURATION] fraction: ulong => 0    
    //         0x50, 0x00, 0x10, 0x00, // [Data Submessage: SerializedPayload] parameterId_1: short => PID_PARTICIPANT_GUID | length: short => 16
    //         0x05, 0x06, 0x07, 0x08, // [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID] 
    //         0x09, 0x05, 0x01, 0x02, // [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x03, 0x04, 0x0a, 0x0b, // [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x00, 0x00, 0x01, 0xc1, // [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x58, 0x00, 0x04, 0x00, // [Data Submessage: SerializedPayload] parameterId_1: short => PID_BUILTIN_ENDPOINT_SET | length: short => 4
    //         0x15, 0x04, 0x00, 0x00, // [Data Submessage: SerializedPayload: PID_BUILTIN_ENDPOINT_SET] BuiltinEndpointSet: bitmask => (0100 0001 0101‬) PARTICIPANT_ANNOUNCER && PUBLICATIONS_ANNOUNCER && SUBSCRIPTIONS_ANNOUNCER && PARTICIPANT_MESSAGE_DATA_WRITER
    //         0x01, 0x00, 0x00, 0x00, // [Data Submessage: SerializedPayload] parameterId_1: short => PID_SENTINEL |  length: short => 0
    //     ];
    //     assert_eq!(
    //         cdr::serialize::<_,_,PlCdrLe>(&participant, Infinite).unwrap(),
    //         data);
    // }
}
