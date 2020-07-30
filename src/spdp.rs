use std::io::Write;
use crate::messages::Endianness;

use crate::types::{VendorId, Locator, ProtocolVersion, };
use crate::messages::{ParameterList, };
use crate::messages::SubmessageElement;

use crate::endpoint_types::{
    DomainId,
    ParameterDomainId,
    ParameterDomainTag,
    ParameterProtocolVersion,
    ParameterVendorId,
    ParameterExpectsInlineQoS,
    ParameterMetatrafficUnicastLocator, };



#[derive(Debug, PartialEq)]
pub struct SpdpParticipantData{
    domain_id: DomainId,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    // guid_prefix: GuidPrefix, // Implicit by the key (TODO)
    vendor_id: VendorId,
    expects_inline_qos: bool,
    metatraffic_unicast_locator_list: Vec<Locator>,
}

impl SpdpParticipantData {
    fn serialize(&self, writer: &mut impl Write, endianness: Endianness) {

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

        parameter_list.serialize(writer, endianness).unwrap();
    }

    fn deserialize(bytes: &[u8]) -> Self {
        if bytes.len() < 4 {
            panic!("Message too small");
        }

        let endianness = match &bytes[0..4] {
            &[0x00, 0x02, 0x00, 0x00] => Endianness::BigEndian,
            &[0x00, 0x03, 0x00, 0x00] => Endianness::LittleEndian,
            _ => panic!("Invalid header"),
        };

        let parameter_list = ParameterList::deserialize(&bytes[4..], endianness).unwrap();
        let domain_id = parameter_list.find::<ParameterDomainId>(endianness).unwrap().0;
        let domain_tag = parameter_list.find::<ParameterDomainTag>(endianness).unwrap_or_default().0;
        let protocol_version = parameter_list.find::<ParameterProtocolVersion>(endianness).unwrap().0;
        let vendor_id = parameter_list.find::<ParameterVendorId>(endianness).unwrap().0;
        let expects_inline_qos = parameter_list.find::<ParameterExpectsInlineQoS>(endianness).unwrap_or_default().0;
        let metatraffic_unicast_locator_list = 
            parameter_list.find_all::<ParameterMetatrafficUnicastLocator>(endianness).unwrap()
            .iter().map(|x|x.0).collect();

        SpdpParticipantData{
            domain_id,
            domain_tag,
            protocol_version,
            vendor_id,
            expects_inline_qos,
            metatraffic_unicast_locator_list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::PROTOCOL_VERSION_2_4;

    #[test]
    fn serialize_spdp_data() {
        let spdp_participant_data = SpdpParticipantData{
            domain_id: 1,
            domain_tag: "abcd".to_string(),
            protocol_version: PROTOCOL_VERSION_2_4,
            vendor_id: [99,99],
            expects_inline_qos: true,
            metatraffic_unicast_locator_list: vec![ Locator::new(10,100,[1;16]), Locator::new(5,20000,[20;16])],
        };

        let mut bytes = Vec::new();

        spdp_participant_data.serialize(&mut bytes, Endianness::BigEndian);
        println!("Result: {:?}", bytes);

        let deserialized_spdp = SpdpParticipantData::deserialize(&bytes);
        println!("Deserialized Result: {:?}", deserialized_spdp);
        assert_eq!(deserialized_spdp,spdp_participant_data);
    }

}