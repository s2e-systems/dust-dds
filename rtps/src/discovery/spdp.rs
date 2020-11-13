use std::sync::{Arc, Mutex};
use std::convert::TryInto;
use rust_dds_interface::types::{TopicKind, DomainId, InstanceHandle, ChangeKind};
use rust_dds_interface::history_cache::HistoryCache;

use crate::types::{GuidPrefix, GUID, Locator, ProtocolVersion, VendorId, ReliabilityKind};
use crate::behavior::{StatelessWriter, StatelessReader};
use super::super::behavior::stateless_reader::StatelessReaderListener;

use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR};
use crate::messages::types::Count;
use crate::behavior::types::Duration;

use crate::endpoint_types::{
    BuiltInEndpointSet,
    // ParameterDomainId,
    // ParameterDomainTag,
    // ParameterProtocolVersion,
    // ParameterVendorId,
    // ParameterExpectsInlineQoS,
    // ParameterMetatrafficUnicastLocator, 
    // ParameterMetatrafficMulticastLocator, 
    // ParameterDefaultUnicastLocator, 
    // ParameterDefaultMulticastLocator,
    // ParameterBuiltInEndpointSet, 
    // ParameterParticipantLeaseDuration,
    // ParameterParticipantManualLivelinessCount, 
    };


pub struct SimpleParticipantDiscoveryProtocol {
    spdp_builtin_participant_writer: Arc<StatelessWriter>,
    spdp_builtin_participant_reader: Arc<StatelessReader>,
}

impl SimpleParticipantDiscoveryProtocol {
    pub fn new(spdp_data: SPDPdiscoveredParticipantData, spdp_listener: impl StatelessReaderListener) -> Self {
        let writer_guid = GUID::new(spdp_data.guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        let writer_cache = HistoryCache::default();
        let mut spdp_builtin_participant_writer = StatelessWriter::new(
            writer_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            writer_cache,
            );

        // let change = spdp_builtin_participant_writer.new_change(ChangeKind::Alive, Some(spdp_data.data(CdrEndianness::LittleEndian)), None, spdp_data.key());
        // spdp_builtin_participant_writer.writer_cache().add_change(change).unwrap();

        for locator in &spdp_data.metatraffic_multicast_locator_list {
            spdp_builtin_participant_writer.reader_locator_add(locator.clone());
        }

        let reader_guid = GUID::new(spdp_data.guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR);
        let reader_cache = HistoryCache::default();

        let spdp_builtin_participant_reader = StatelessReader::new(
            reader_guid,
            TopicKind::WithKey, 
            ReliabilityKind::BestEffort,
            vec![],
            spdp_data.metatraffic_multicast_locator_list.clone(),
            false,
            reader_cache,
            spdp_listener
        ); 

        Self {
            spdp_builtin_participant_writer: Arc::new(spdp_builtin_participant_writer),
            spdp_builtin_participant_reader: Arc::new(spdp_builtin_participant_reader),
        }        
    }

    pub fn spdp_builtin_participant_writer(&self) -> &Arc<StatelessWriter> {
        &self.spdp_builtin_participant_writer
    }

    pub fn spdp_builtin_participant_reader(&self) -> &Arc<StatelessReader> {
        &self.spdp_builtin_participant_reader
    }

}


#[derive(Debug, PartialEq)]
pub struct SPDPdiscoveredParticipantData{
    domain_id: DomainId,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    guid_prefix: GuidPrefix,
    vendor_id: VendorId,
    expects_inline_qos: bool,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    available_built_in_endpoints: BuiltInEndpointSet,
    lease_duration: Duration,
    manual_liveliness_count: Count,
    // built_in_endpoint_qos: BuiltInEndpointQos
}

impl SPDPdiscoveredParticipantData {
    pub fn new(
        domain_id: DomainId,
        domain_tag: String,
        protocol_version: ProtocolVersion,
        guid_prefix: GuidPrefix,
        vendor_id: VendorId,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        available_built_in_endpoints: BuiltInEndpointSet,
        lease_duration: Duration) -> Self{
        Self {
            domain_id,
            domain_tag,
            protocol_version,
            guid_prefix,
            vendor_id,
            expects_inline_qos: false, // TODO
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            available_built_in_endpoints,
            lease_duration,
            manual_liveliness_count: 0, //TODO:Count,
        }
    }

    pub fn domain_id(&self) -> DomainId{
        self.domain_id
    }

    pub fn domain_tag(&self) -> &String {
        &self.domain_tag
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &Vec<Locator> {
        &self.metatraffic_unicast_locator_list
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &Vec<Locator> {
        &self.metatraffic_multicast_locator_list
    }

    pub fn default_unicast_locator_list(&self) -> &Vec<Locator> {
        &self.default_unicast_locator_list
    }

    pub fn default_multicast_locator_list(&self) -> &Vec<Locator> {
        &self.default_multicast_locator_list
    }

    pub fn available_built_in_endpoints(&self) -> &BuiltInEndpointSet {
        &self.available_built_in_endpoints
    }

    pub fn key(&self) -> InstanceHandle {
        let mut instance_handle = [0;16];
        instance_handle[0..12].copy_from_slice(&self.guid_prefix);
        instance_handle
    }

    pub fn data(&self) -> Vec<u8> {
        todo!()
        // let mut parameter_list = CdrParameterList::new(endianness);

        // // Defaults to the domainId of the local participant receiving the message
        // // TODO: Add the chance of sending a specific domain_id
        // // parameter_list.push(ParameterDomainId(self.domain_id));

        // if self.domain_tag != ParameterDomainTag::default() {
        //     parameter_list.push(ParameterDomainTag(self.domain_tag.clone()));
        // }

        // parameter_list.push(ParameterProtocolVersion(self.protocol_version));

        // parameter_list.push(ParameterVendorId(self.vendor_id));

        // if self.expects_inline_qos != ParameterExpectsInlineQoS::default() {
        //     parameter_list.push(ParameterExpectsInlineQoS(self.expects_inline_qos));
        // }

        // for metatraffic_unicast_locator in &self.metatraffic_unicast_locator_list {
        //     parameter_list.push(ParameterMetatrafficUnicastLocator(*metatraffic_unicast_locator));
        // }

        // for metatraffic_multicast_locator in &self.metatraffic_multicast_locator_list {
        //     parameter_list.push(ParameterMetatrafficMulticastLocator(*metatraffic_multicast_locator));
        // }

        // for default_unicast_locator in &self.default_unicast_locator_list {
        //     parameter_list.push(ParameterDefaultUnicastLocator(*default_unicast_locator));
        // }

        // for default_multicast_locator in &self.default_multicast_locator_list {
        //     parameter_list.push(ParameterDefaultMulticastLocator(*default_multicast_locator));
        // }

        // parameter_list.push(ParameterBuiltInEndpointSet(self.available_built_in_endpoints));

        // if self.lease_duration != ParameterParticipantLeaseDuration::default() {
        //     parameter_list.push(ParameterParticipantLeaseDuration(self.lease_duration));
        // }

        // parameter_list.push(ParameterParticipantManualLivelinessCount(self.manual_liveliness_count));

        // parameter_list.as_bytes()
    }

    pub fn from_key_data(key: InstanceHandle, data: &[u8], default_domain_id: DomainId) -> Self {
        todo!()
        // let guid_prefix: GuidPrefix = key[0..12].try_into().unwrap();

        // let parameter_list = CdrParameterList::from_bytes(&data);

        // let domain_id = parameter_list.find::<ParameterDomainId>().unwrap_or(ParameterDomainId(default_domain_id)).0;
        // let domain_tag = parameter_list.find::<ParameterDomainTag>().unwrap_or_default().0;
        // let protocol_version = parameter_list.find::<ParameterProtocolVersion>().unwrap().0;
        // let vendor_id = parameter_list.find::<ParameterVendorId>().unwrap().0;
        // let expects_inline_qos = parameter_list.find::<ParameterExpectsInlineQoS>().unwrap_or_default().0;
        // let metatraffic_unicast_locator_list = 
        //     parameter_list.find_all::<ParameterMetatrafficUnicastLocator>()
        //     .iter().map(|x|x.0).collect();
        // let metatraffic_multicast_locator_list = 
        //     parameter_list.find_all::<ParameterMetatrafficMulticastLocator>()
        //     .iter().map(|x|x.0).collect();
        // let default_unicast_locator_list = 
        //     parameter_list.find_all::<ParameterDefaultUnicastLocator>()
        //     .iter().map(|x|x.0).collect();
        // let default_multicast_locator_list = 
        //     parameter_list.find_all::<ParameterDefaultMulticastLocator>()
        //     .iter().map(|x|x.0).collect();
        // let available_built_in_endpoints = parameter_list.find::<ParameterBuiltInEndpointSet>().unwrap().0;
        // let lease_duration = parameter_list.find::<ParameterParticipantLeaseDuration>().unwrap_or_default().0;
        // let manual_liveliness_count = parameter_list.find::<ParameterParticipantManualLivelinessCount>().unwrap().0;

        // Self{
        //     domain_id,
        //     domain_tag,
        //     protocol_version,
        //     guid_prefix,
        //     vendor_id,
        //     expects_inline_qos,
        //     metatraffic_unicast_locator_list,
        //     metatraffic_multicast_locator_list,
        //     default_unicast_locator_list,
        //     default_multicast_locator_list,
        //     available_built_in_endpoints,
        //     lease_duration,
        //     manual_liveliness_count,
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::transport::udp::UdpTransport;
    use crate::types::constants::PROTOCOL_VERSION_2_4;
    // struct MockTransport{
    //     sent_messages: Vec<RtpsMessage>,
    //     unicast_locator_list: Vec<Locator>,
    //     multicast_locator_list: Vec<Locator>,
    // }

    // impl MockTransport{
    //     fn new() -> Self {
    //         Self {
    //             sent_messages: Vec::new(),
    //             unicast_locator_list: vec![Locator::new_udpv4(7400, [127,0,0,1])],
    //             multicast_locator_list: vec![Locator::new_udpv4(7400, [239,255,0,1])],
    //         }
    //     }
    // }

    // impl Transport for MockTransport {
    //     fn write(&self, message: RtpsMessage, destination_locator: &Locator) {
    //         println!("Sent message: {:?}", message);
    //     }

    //     fn read(&self) -> crate::transport::TransportResult<Option<(RtpsMessage, Locator)>> {
    //         let endianness = Endianness::LittleEndian;
    //         let time = Time::new(100000, 10);
    //         let info_ts_submessage = InfoTs::new(endianness, Some(time));

    //         let reader_id = ENTITYID_UNKNOWN;
    //         let writer_id = ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;
    //         let writer_sn = 1;
    //         let mut inline_qos = ParameterList::new();
    //         inline_qos.push(StatusInfo([0,0,0,0])); // Alive status 
    //         inline_qos.push(KeyHash([10;16]));
    //         let payload = Payload::Data(vec![0,0,0,0]);
    //         let data = Data::new(endianness,
    //             reader_id,
    //             writer_id,
    //             writer_sn,
    //             Some(inline_qos),
    //             payload);

    //         let guid_prefix = [5;12];
    //         let submessages = vec![RtpsSubmessage::InfoTs(info_ts_submessage), RtpsSubmessage::Data(data)];

    //         let message = RtpsMessage::new(PROTOCOL_VERSION_2_4, [99,99], guid_prefix, submessages);
    //         let locator = Locator::new_udpv4(7400, [239,255,0,1]);
    //         Ok(Some((message, locator)))
    //     }

    //     fn unicast_locator_list(&self) -> &Vec<Locator> {
    //         &self.unicast_locator_list
    //     }

    //     fn multicast_locator_list(&self) -> &Vec<Locator> {
    //         &self.multicast_locator_list
    //     }

    //     fn as_any(&self) -> &dyn std::any::Any {
    //         todo!()
    //     }

        
    // }

    // #[test]
    // fn discovery_send() {
    //     // Left here for debugging
    //     // let userdata_transport = UdpTransport::default_userdata_transport(0, "Wi-Fi").unwrap();
    //     // let metatraffic_transport = UdpTransport::default_metatraffic_transport(0, "Wi-Fi").unwrap();
    //     let userdata_transport = MockTransport::new();
    //     let metatraffic_transport = MockTransport::new();
    //     let participant = Arc::new(RtpsParticipant::new(0, userdata_transport, metatraffic_transport));

    //     let mut spdp = SPDP::new(&participant);

    //     spdp.send();
    // }

    // #[test]
    // fn discovery_receive() {
    //     let userdata_transport = MockTransport::new();
    //     let metatraffic_transport = MockTransport::new();
    //     let participant = Arc::new(RtpsParticipant::new(0, userdata_transport, metatraffic_transport));

    //     let mut spdp = SPDP::new(&participant);

    //     spdp.receive();
    // }

    // #[test]
    // fn complete_serialize_spdp_data() {
    //     let spdp_participant_data = SPDPdiscoveredParticipantData{
    //         domain_id: 0,
    //         domain_tag: "abcd".to_string(),
    //         protocol_version: PROTOCOL_VERSION_2_4,
    //         guid_prefix: [1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5],
    //         vendor_id: [99,99],
    //         expects_inline_qos: true,
    //         metatraffic_unicast_locator_list: vec![ Locator::new(10,100,[1;16]) ],
    //         metatraffic_multicast_locator_list: vec![ Locator::new(20,100,[5;16]), Locator::new(5,2300,[30;16])],
    //         default_unicast_locator_list: vec![ Locator::new(10,100,[1;16]), Locator::new(5,20000,[20;16])],
    //         default_multicast_locator_list: vec![ Locator::new(50,100,[9;16]), Locator::new(5,1300,[30;16]), Locator::new(555,1300,[30;16])],
    //         available_built_in_endpoints: BuiltInEndpointSet::new(123),
    //         lease_duration: Duration::from_secs(30),
    //         manual_liveliness_count: 0,
    //     };

    //     let key = spdp_participant_data.key();

    //     assert_eq!(key,  [1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 0, 0, 0, 0]);

    //     let data = spdp_participant_data.data(CdrEndianness::BigEndian);
    //     assert_eq!(data, 
    //         [0, 2, 0, 0, // CDR_PL_BE
    //         // 0, 15, 0, 4, // PID: 0x000f (PID_DOMAIN_ID) Length: 4
    //         // 0, 0, 0, 1,  // DomainId
    //         64, 20, 0, 12, // PID: 0x4014 (PID_DOMAIN_TAG) Length: 12
    //         0, 0, 0, 5, 97, 98, 99, 100, 0, 0, 0, 0, // DomainTag
    //         0, 21, 0, 4, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
    //         2, 4, 0, 0, // ProtocolVersion
    //         0, 22, 0, 4, // PID: 0x0016 (PID_VENDORID) Length: 4
    //         99, 99, 0, 0, //VendorId
    //         0, 67, 0, 4, // PID: 0x0043 (PID_EXPECTS_INLINE_QOS) Length: 4
    //         1, 0, 0, 0, //Bool
    //         0, 50, 0, 24, // PID:0x0032 (PID_METATRAFFIC_UNICAST_LOCATOR) Length: 24
    //         0, 0, 0, 10, 0, 0, 0, 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
    //         0, 51, 0, 24, // PID:0x0033 (PID_METATRAFFIC_MULTICAST_LOCATOR) Length: 24
    //         0, 0, 0, 20, 0, 0, 0, 100, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, // Locator
    //         0, 51, 0, 24, // PID:0x0033 (PID_METATRAFFIC_MULTICAST_LOCATOR) Length: 24
    //         0, 0, 0, 5, 0, 0, 8, 252, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
    //         0, 49, 0, 24, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
    //         0, 0, 0, 10, 0, 0, 0, 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
    //         0, 49, 0, 24, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
    //         0, 0, 0, 5, 0, 0, 78, 32, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, // Locator
    //         0, 72, 0, 24, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
    //         0, 0, 0, 50, 0, 0, 0, 100, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, // Locator
    //         0, 72, 0, 24, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
    //         0, 0, 0, 5, 0, 0, 5, 20, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
    //         0, 72, 0, 24, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
    //         0, 0, 2, 43, 0, 0, 5, 20, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
    //         0, 88, 0, 4, // PID:0x0058 (PID_BUILTIN_ENDPOINT_SET) Length: 4
    //         0, 0, 0, 123, //BuiltInEndpointSet
    //         0, 2, 0, 8,  // PID:0x0002 (PID_PARTICIPANT_LEASE_DURATION) Length: 8
    //         0, 0, 0, 30, 0, 0, 0, 0, // Duration
    //         0, 52, 0, 4, // PID:0x0034 (PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT) Length: 8
    //         0, 0, 0, 0, // Count
    //         0, 1, 0, 0 // PID_SENTINEL
    //     ].to_vec());

    //     let deserialized_spdp = SPDPdiscoveredParticipantData::from_key_data(key, &data, 0);
    //     assert_eq!(deserialized_spdp,spdp_participant_data);

    //     let data = spdp_participant_data.data(CdrEndianness::LittleEndian);
    //     assert_eq!(data, 
    //         [0, 3, 0, 0, // CDR_PL_BE
    //         // 15, 0, 4, 0, // PID: 0x000f (PID_DOMAIN_ID) Length: 4
    //         // 1, 0, 0, 0,  // DomainId
    //         20, 64, 12, 0, // PID: 0x4014 (PID_DOMAIN_TAG) Length: 12
    //         5, 0, 0, 0, 97, 98, 99, 100, 0, 0, 0, 0, // DomainTag
    //         21, 0, 4, 0, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
    //         2, 4, 0, 0, // ProtocolVersion
    //         22, 0, 4, 0, // PID: 0x0016 (PID_VENDORID) Length: 4
    //         99, 99, 0, 0, //VendorId
    //         67, 0, 4, 0, // PID: 0x0043 (PID_EXPECTS_INLINE_QOS) Length: 4
    //         1, 0, 0, 0, //Bool
    //         50, 0, 24, 0, // PID:0x0032 (PID_METATRAFFIC_UNICAST_LOCATOR) Length: 24
    //         10, 0, 0, 0, 100, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
    //         51, 0, 24, 0, // PID:0x0033 (PID_METATRAFFIC_MULTICAST_LOCATOR) Length: 24
    //         20, 0, 0, 0, 100, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, // Locator
    //         51, 0, 24, 0, // PID:0x0033 (PID_METATRAFFIC_MULTICAST_LOCATOR) Length: 24
    //         5, 0, 0, 0, 252, 8, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
    //         49, 0, 24, 0, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
    //         10, 0, 0, 0, 100, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
    //         49, 0, 24, 0, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
    //         5, 0, 0, 0, 32, 78, 0, 0, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, // Locator
    //         72, 0, 24, 0, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
    //         50, 0, 0, 0, 100, 0, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, // Locator
    //         72, 0, 24, 0, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
    //         5, 0, 0, 0, 20, 5, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
    //         72, 0, 24, 0, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
    //         43, 2, 0, 0, 20, 5, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
    //         88, 0, 4, 0, // PID:0x0058 (PID_BUILTIN_ENDPOINT_SET) Length: 4
    //         123, 0, 0, 0, //BuiltInEndpointSet
    //         2, 0, 8, 0,  // PID:0x0002 (PID_PARTICIPANT_LEASE_DURATION) Length: 8
    //         30, 0, 0, 0,0, 0, 0, 0, // Duration
    //         52, 0,  4, 0,// PID:0x0034 (PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT) Length: 8
    //         0, 0, 0, 0, // Count
    //         1, 0, 0, 0 // PID_SENTINEL
    //     ].to_vec());

    //     let deserialized_spdp = SPDPdiscoveredParticipantData::from_key_data(key, &data, 0);
    //     assert_eq!(deserialized_spdp,spdp_participant_data);
    // }

    // #[test]
    // fn serialize_spdp_data_with_defaults() {
    //     let spdp_participant_data = SPDPdiscoveredParticipantData{
    //         domain_id: 0,
    //         domain_tag: "".to_string(),
    //         protocol_version: PROTOCOL_VERSION_2_4,
    //         vendor_id: [99,99],
    //         guid_prefix: [1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5],
    //         expects_inline_qos: false,
    //         metatraffic_unicast_locator_list: vec![],
    //         metatraffic_multicast_locator_list: vec![],
    //         default_unicast_locator_list: vec![Locator::new(10,100,[1;16])],
    //         default_multicast_locator_list: vec![],
    //         available_built_in_endpoints: BuiltInEndpointSet::new(123),
    //         lease_duration: Duration::from_secs(100),
    //         manual_liveliness_count: 0,
    //     };

    //     let key = spdp_participant_data.key();
        
    //     let data = spdp_participant_data.data(CdrEndianness::BigEndian);
    //     assert_eq!(data, 
    //         [0, 2, 0, 0, // CDR_PL_BE
    //         // 0, 15, 0, 4, // PID: 0x00f (PID_DOMAIN_ID) Length: 4
    //         // 0, 0, 0, 1,  // DomainId
    //         0, 21, 0, 4, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
    //         2, 4, 0, 0, // ProtocolVersion
    //         0, 22, 0, 4, // PID: 0x0016 (PID_VENDORID) Length: 4
    //         99, 99, 0, 0, //VendorId
    //         0, 49, 0, 24, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
    //         0, 0, 0, 10, 0, 0, 0, 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
    //         0, 88, 0, 4, // PID:0x0058 (PID_BUILTIN_ENDPOINT_SET) Length: 4
    //         0, 0, 0, 123, //BuiltInEndpointSet
    //         0, 52, 0, 4, // PID:0x0034 (PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT) Length: 8
    //         0, 0, 0, 0, // Count
    //         0, 1, 0, 0 // PID_SENTINEL
    //     ].to_vec());

    //     let deserialized_spdp = SPDPdiscoveredParticipantData::from_key_data(key, &data, 0);
    //     assert_eq!(deserialized_spdp,spdp_participant_data);
    // }
}