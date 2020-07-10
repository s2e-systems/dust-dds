use crate::serdes::{SubmessageElement, Endianness, RtpsSerdesResult, };

use super::types::{SubmessageKind, SubmessageFlag, };
use super::{SubmessageHeader, Submessage, };
use super::{submessage_elements,};
use super::submessage_elements::Long;

#[derive(PartialEq, Debug)]
pub struct InfoSource {
    endianness_flag: SubmessageFlag,
    protocol_version: submessage_elements::ProtocolVersion,
    vendor_id: submessage_elements::VendorId,
    guid_prefix: submessage_elements::GuidPrefix,
}


impl Submessage for InfoSource {
    fn submessage_header(&self) -> SubmessageHeader {
        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        let flags = [e, X, X, X, X, X, X, X];
        
        let octets_to_next_header = 
            4 /*unused.octets()*/ +
            self.protocol_version.octets() + 
            self.vendor_id.octets() +
            self.guid_prefix.octets();

        SubmessageHeader { 
            submessage_id: SubmessageKind::InfoSource,
            flags,
            submessage_length: octets_to_next_header as u16,
        }
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = self.endianness_flag.into();
        let unused = Long(0);
        self.submessage_header().compose(writer)?;
        unused.serialize(writer, endianness)?;
        self.protocol_version.serialize(writer, endianness)?;
        self.vendor_id.serialize(writer, endianness)?;
        self.guid_prefix.serialize(writer, endianness)?;
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let endianness = Endianness::from(endianness_flag);
        let _unused = Long::deserialize(&bytes[4..8], endianness)?;
        let protocol_version = submessage_elements::ProtocolVersion::deserialize(&bytes[8..10], endianness)?;
        let vendor_id = submessage_elements::VendorId::deserialize(&bytes[10..12], endianness)?;
        let guid_prefix = submessage_elements::GuidPrefix::deserialize(&bytes[12..24], endianness)?;        

        Ok(InfoSource {
            endianness_flag,
            protocol_version,
            vendor_id,
            guid_prefix,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants;

    #[test]
    fn parse_heartbeat_frag_submessage() {
        let expected = InfoSource {
            endianness_flag: true,    
            protocol_version: submessage_elements::ProtocolVersion(constants::PROTOCOL_VERSION_2_4),
            vendor_id: submessage_elements::VendorId(constants::VENDOR_ID),
            guid_prefix: submessage_elements::GuidPrefix([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        };
        let bytes = vec![
            0x0c, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00, 0x00, 0x00, // unused
             2,  4, 99, 99, // protocol_version | vendor_id
             1,  2,  3,  4, // guid_prefix
             5,  6,  7,  8, // guid_prefix 
             9, 10, 11, 12, // guid_prefix
        ];
        let result = InfoSource::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    
    #[test]
    fn compose_heartbeat_frag_submessage() {
        let message = InfoSource {
            endianness_flag: true,    
            protocol_version: submessage_elements::ProtocolVersion(constants::PROTOCOL_VERSION_2_4),
            vendor_id: submessage_elements::VendorId(constants::VENDOR_ID),
            guid_prefix: submessage_elements::GuidPrefix([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        };
        let expected = vec![
            0x0c, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00, 0x00, 0x00, // unused
             2,  4, 99, 99, // protocol_version | vendor_id
             1,  2,  3,  4, // guid_prefix
             5,  6,  7,  8, // guid_prefix 
             9, 10, 11, 12, // guid_prefix
        ];
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }

}
