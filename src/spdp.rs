use std::io::Write;
use std::convert::TryInto;
use crate::messages::Endianness;

use crate::types::{VendorId, Locator, ProtocolVersion, GuidPrefix, InstanceHandle};
use crate::messages::{ParameterList, SubmessageElement};
use crate::messages::types::Count;
use crate::behavior::types::Duration;
use crate::participant::Participant;

use crate::endpoint_types::{
    DomainId,
    BuiltInEndpointSet,
    ParameterDomainId,
    ParameterDomainTag,
    ParameterProtocolVersion,
    ParameterVendorId,
    ParameterExpectsInlineQoS,
    ParameterMetatrafficUnicastLocator, 
    ParameterMetatrafficMulticastLocator, 
    ParameterDefaultUnicastLocator, 
    ParameterDefaultMulticastLocator,
    ParameterBuiltInEndpointSet, 
    ParameterParticipantLeaseDuration,
    ParameterParticipantManualLivelinessCount, 
    };



#[derive(Debug, PartialEq)]
pub struct SPDPdiscoveredParticipantData{
    domain_id: DomainId,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    guid_prefix: GuidPrefix, // Implicit by the key (TODO)
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
    fn new_from_participant(participant: &Participant, lease_duration: Duration) -> Self{
        Self {
            domain_id: 0, //TODO: participant.domain_id(),
            domain_tag: "".to_string(), //TODO: participant.domain_tag(),
            protocol_version: participant.protocol_version(),
            guid_prefix: *participant.guid().prefix(),
            vendor_id: participant.vendor_id(),
            expects_inline_qos: false, // TODO
            metatraffic_unicast_locator_list: vec![], //TODO: participant.metatraffic_unicast_locator_list().clone(),
            metatraffic_multicast_locator_list: vec![], //TODO: participant.metatraffic_multicast_locator_list().clone(),
            default_unicast_locator_list: participant.default_unicast_locator_list().clone(),
            default_multicast_locator_list: participant.default_multicast_locator_list().clone(),
            available_built_in_endpoints: BuiltInEndpointSet::new(0),
            lease_duration,
            manual_liveliness_count: 0, //TODO:Count,
        }
    }

    fn key(&self) -> InstanceHandle {
        let mut instance_handle = [0;16];
        instance_handle[0..12].copy_from_slice(&self.guid_prefix);
        instance_handle
    }

    fn data(&self, writer: &mut impl Write, endianness: Endianness) {

        // Start by writing the header which depends on the endianness
        match endianness {
            Endianness::BigEndian => writer.write(&[0x00, 0x02, 0x00, 0x00]),
            Endianness::LittleEndian => writer.write(&[0x00, 0x03, 0x00, 0x00]),
        }.unwrap();

        let mut parameter_list = ParameterList::new();

        parameter_list.push(ParameterDomainId(self.domain_id));

        if self.domain_tag != ParameterDomainTag::default() {
            parameter_list.push(ParameterDomainTag(self.domain_tag.clone()));
        }

        parameter_list.push(ParameterProtocolVersion(self.protocol_version));

        parameter_list.push(ParameterVendorId(self.vendor_id));

        if self.expects_inline_qos != ParameterExpectsInlineQoS::default() {
            parameter_list.push(ParameterExpectsInlineQoS(self.expects_inline_qos));
        }

        for metatraffic_unicast_locator in &self.metatraffic_unicast_locator_list {
            parameter_list.push(ParameterMetatrafficUnicastLocator(*metatraffic_unicast_locator));
        }

        for metatraffic_multicast_locator in &self.metatraffic_multicast_locator_list {
            parameter_list.push(ParameterMetatrafficMulticastLocator(*metatraffic_multicast_locator));
        }

        for default_unicast_locator in &self.default_unicast_locator_list {
            parameter_list.push(ParameterDefaultUnicastLocator(*default_unicast_locator));
        }

        for default_multicast_locator in &self.default_multicast_locator_list {
            parameter_list.push(ParameterDefaultMulticastLocator(*default_multicast_locator));
        }

        parameter_list.push(ParameterBuiltInEndpointSet(self.available_built_in_endpoints));

        if self.lease_duration != ParameterParticipantLeaseDuration::default() {
            parameter_list.push(ParameterParticipantLeaseDuration(self.lease_duration));
        }

        parameter_list.push(ParameterParticipantManualLivelinessCount(self.manual_liveliness_count));
        

        parameter_list.serialize(writer, endianness).unwrap();
    }

    fn from_key_data(key: InstanceHandle, data: &[u8]) -> Self {
        if data.len() < 4 {
            panic!("Message too small");
        }

        let endianness = match &data[0..4] {
            &[0x00, 0x02, 0x00, 0x00] => Endianness::BigEndian,
            &[0x00, 0x03, 0x00, 0x00] => Endianness::LittleEndian,
            _ => panic!("Invalid header"),
        };

        let guid_prefix: GuidPrefix = key[0..12].try_into().unwrap();
        let parameter_list = ParameterList::deserialize(&data[4..], endianness).unwrap();
        let domain_id = parameter_list.find::<ParameterDomainId>(endianness).unwrap().0;
        let domain_tag = parameter_list.find::<ParameterDomainTag>(endianness).unwrap_or_default().0;
        let protocol_version = parameter_list.find::<ParameterProtocolVersion>(endianness).unwrap().0;
        let vendor_id = parameter_list.find::<ParameterVendorId>(endianness).unwrap().0;
        let expects_inline_qos = parameter_list.find::<ParameterExpectsInlineQoS>(endianness).unwrap_or_default().0;
        let metatraffic_unicast_locator_list = 
            parameter_list.find_all::<ParameterMetatrafficUnicastLocator>(endianness).unwrap()
            .iter().map(|x|x.0).collect();
        let metatraffic_multicast_locator_list = 
            parameter_list.find_all::<ParameterMetatrafficMulticastLocator>(endianness).unwrap()
            .iter().map(|x|x.0).collect();
        let default_unicast_locator_list = 
            parameter_list.find_all::<ParameterDefaultUnicastLocator>(endianness).unwrap()
            .iter().map(|x|x.0).collect();
        let default_multicast_locator_list = 
            parameter_list.find_all::<ParameterDefaultMulticastLocator>(endianness).unwrap()
            .iter().map(|x|x.0).collect();
        let available_built_in_endpoints = parameter_list.find::<ParameterBuiltInEndpointSet>(endianness).unwrap().0;
        let lease_duration = parameter_list.find::<ParameterParticipantLeaseDuration>(endianness).unwrap_or_default().0;
        let manual_liveliness_count = parameter_list.find::<ParameterParticipantManualLivelinessCount>(endianness).unwrap().0;

        Self{
            domain_id,
            domain_tag,
            protocol_version,
            guid_prefix,
            vendor_id,
            expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            available_built_in_endpoints,
            lease_duration,
            manual_liveliness_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::PROTOCOL_VERSION_2_4;

    #[test]
    fn complete_serialize_spdp_data() {
        let spdp_participant_data = SPDPdiscoveredParticipantData{
            domain_id: 1,
            domain_tag: "abcd".to_string(),
            protocol_version: PROTOCOL_VERSION_2_4,
            guid_prefix: [1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5],
            vendor_id: [99,99],
            expects_inline_qos: true,
            metatraffic_unicast_locator_list: vec![ Locator::new(10,100,[1;16]) ],
            metatraffic_multicast_locator_list: vec![ Locator::new(20,100,[5;16]), Locator::new(5,2300,[30;16])],
            default_unicast_locator_list: vec![ Locator::new(10,100,[1;16]), Locator::new(5,20000,[20;16])],
            default_multicast_locator_list: vec![ Locator::new(50,100,[9;16]), Locator::new(5,1300,[30;16]), Locator::new(555,1300,[30;16])],
            available_built_in_endpoints: BuiltInEndpointSet::new(123),
            lease_duration: Duration::from_secs(30),
            manual_liveliness_count: 0,
        };

        let key = spdp_participant_data.key();

        assert_eq!(key,  [1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 0, 0, 0, 0]);

        let mut data = Vec::new();

        spdp_participant_data.data(&mut data, Endianness::BigEndian);
        assert_eq!(data, 
            [0, 2, 0, 0, // CDR_PL_BE
            0, 15, 0, 4, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
            0, 0, 0, 1,  // DomainId
            64, 20, 0, 12, // PID: 0x4014 (PID_DOMAIN_TAG) Length: 12
            0, 0, 0, 5, 97, 98, 99, 100, 0, 0, 0, 0, // DomainTag
            0, 21, 0, 4, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
            2, 4, 0, 0, // ProtocolVersion
            0, 22, 0, 4, // PID: 0x0016 (PID_VENDORID) Length: 4
            99, 99, 0, 0, //VendorId
            0, 67, 0, 4, // PID: 0x0043 (PID_EXPECTS_INLINE_QOS) Length: 4
            1, 0, 0, 0, //Bool
            0, 50, 0, 24, // PID:0x0032 (PID_METATRAFFIC_UNICAST_LOCATOR) Length: 24
            0, 0, 0, 10, 0, 0, 0, 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
            0, 51, 0, 24, // PID:0x0033 (PID_METATRAFFIC_MULTICAST_LOCATOR) Length: 24
            0, 0, 0, 20, 0, 0, 0, 100, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, // Locator
            0, 51, 0, 24, // PID:0x0033 (PID_METATRAFFIC_MULTICAST_LOCATOR) Length: 24
            0, 0, 0, 5, 0, 0, 8, 252, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
            0, 49, 0, 24, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
            0, 0, 0, 10, 0, 0, 0, 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
            0, 49, 0, 24, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
            0, 0, 0, 5, 0, 0, 78, 32, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, // Locator
            0, 72, 0, 24, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
            0, 0, 0, 50, 0, 0, 0, 100, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, // Locator
            0, 72, 0, 24, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
            0, 0, 0, 5, 0, 0, 5, 20, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
            0, 72, 0, 24, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
            0, 0, 2, 43, 0, 0, 5, 20, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
            0, 88, 0, 4, // PID:0x0058 (PID_BUILTIN_ENDPOINT_SET) Length: 4
            0, 0, 0, 123, //BuiltInEndpointSet
            0, 2, 0, 8,  // PID:0x0002 (PID_PARTICIPANT_LEASE_DURATION) Length: 8
            0, 0, 0, 30, 0, 0, 0, 0, // Duration
            0, 52, 0, 4, // PID:0x0034 (PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT) Length: 8
            0, 0, 0, 0, // Count
            0, 1, 0, 0 // PID_SENTINEL
        ].to_vec());

        let deserialized_spdp = SPDPdiscoveredParticipantData::from_key_data(key, &data);
        assert_eq!(deserialized_spdp,spdp_participant_data);

        data.clear();

        spdp_participant_data.data(&mut data, Endianness::LittleEndian);
        assert_eq!(data, 
            [0, 3, 0, 0, // CDR_PL_BE
            15, 0, 4, 0, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
            1, 0, 0, 0,  // DomainId
            20, 64, 12, 0, // PID: 0x4014 (PID_DOMAIN_TAG) Length: 12
            5, 0, 0, 0, 97, 98, 99, 100, 0, 0, 0, 0, // DomainTag
            21, 0, 4, 0, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
            2, 4, 0, 0, // ProtocolVersion
            22, 0, 4, 0, // PID: 0x0016 (PID_VENDORID) Length: 4
            99, 99, 0, 0, //VendorId
            67, 0, 4, 0, // PID: 0x0043 (PID_EXPECTS_INLINE_QOS) Length: 4
            1, 0, 0, 0, //Bool
            50, 0, 24, 0, // PID:0x0032 (PID_METATRAFFIC_UNICAST_LOCATOR) Length: 24
            10, 0, 0, 0, 100, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
            51, 0, 24, 0, // PID:0x0033 (PID_METATRAFFIC_MULTICAST_LOCATOR) Length: 24
            20, 0, 0, 0, 100, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, // Locator
            51, 0, 24, 0, // PID:0x0033 (PID_METATRAFFIC_MULTICAST_LOCATOR) Length: 24
            5, 0, 0, 0, 252, 8, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
            49, 0, 24, 0, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
            10, 0, 0, 0, 100, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
            49, 0, 24, 0, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
            5, 0, 0, 0, 32, 78, 0, 0, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, // Locator
            72, 0, 24, 0, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
            50, 0, 0, 0, 100, 0, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, // Locator
            72, 0, 24, 0, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
            5, 0, 0, 0, 20, 5, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
            72, 0, 24, 0, // PID:0x0048 (PID_DEFAULT_MULTICAST_LOCATOR) Length: 24
            43, 2, 0, 0, 20, 5, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, // Locator
            88, 0, 4, 0, // PID:0x0058 (PID_BUILTIN_ENDPOINT_SET) Length: 4
            123, 0, 0, 0, //BuiltInEndpointSet
            2, 0, 8, 0,  // PID:0x0002 (PID_PARTICIPANT_LEASE_DURATION) Length: 8
            30, 0, 0, 0,0, 0, 0, 0, // Duration
            52, 0,  4, 0,// PID:0x0034 (PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT) Length: 8
            0, 0, 0, 0, // Count
            1, 0, 0, 0 // PID_SENTINEL
        ].to_vec());

        let deserialized_spdp = SPDPdiscoveredParticipantData::from_key_data(key, &data);
        assert_eq!(deserialized_spdp,spdp_participant_data);
    }

    #[test]
    fn serialize_spdp_data_with_defaults() {
        let spdp_participant_data = SPDPdiscoveredParticipantData{
            domain_id: 1,
            domain_tag: "".to_string(),
            protocol_version: PROTOCOL_VERSION_2_4,
            vendor_id: [99,99],
            guid_prefix: [1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5],
            expects_inline_qos: false,
            metatraffic_unicast_locator_list: vec![],
            metatraffic_multicast_locator_list: vec![],
            default_unicast_locator_list: vec![Locator::new(10,100,[1;16])],
            default_multicast_locator_list: vec![],
            available_built_in_endpoints: BuiltInEndpointSet::new(123),
            lease_duration: Duration::from_secs(100),
            manual_liveliness_count: 0,
        };

        let key = spdp_participant_data.key();
        let mut data = Vec::new();

        spdp_participant_data.data(&mut data, Endianness::BigEndian);
        assert_eq!(data, 
            [0, 2, 0, 0, // CDR_PL_BE
            0, 15, 0, 4, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
            0, 0, 0, 1,  // DomainId
            0, 21, 0, 4, // PID: 0x0015 (PID_PROTOCOL_VERSION) Length: 4
            2, 4, 0, 0, // ProtocolVersion
            0, 22, 0, 4, // PID: 0x0016 (PID_VENDORID) Length: 4
            99, 99, 0, 0, //VendorId
            0, 49, 0, 24, // PID:0x0031 (PID_DEFAULT_UNICAST_LOCATOR) Length: 24
            0, 0, 0, 10, 0, 0, 0, 100, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // Locator
            0, 88, 0, 4, // PID:0x0058 (PID_BUILTIN_ENDPOINT_SET) Length: 4
            0, 0, 0, 123, //BuiltInEndpointSet
            0, 52, 0, 4, // PID:0x0034 (PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT) Length: 8
            0, 0, 0, 0, // Count
            0, 1, 0, 0 // PID_SENTINEL
        ].to_vec());
        
        let deserialized_spdp = SPDPdiscoveredParticipantData::from_key_data(key, &data);
        assert_eq!(deserialized_spdp,spdp_participant_data);
    }

}