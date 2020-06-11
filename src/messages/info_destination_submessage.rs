use crate::primitive_types::UShort;
use crate::types::GuidPrefix;
use crate::messages::types::{SubmessageKind, SubmessageFlag, };
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, Endianness, RtpsSerdesResult, };
use super::{SubmessageHeader, Submessage, };


#[derive(PartialEq, Debug)]
pub struct InfoDestination {
    endianness_flag: SubmessageFlag,
    guid_prefix: GuidPrefix,
}

impl Submessage for InfoDestination {
    fn submessage_header(&self) -> SubmessageHeader {
        const X : SubmessageFlag = false;
        let e = self.endianness_flag; // Indicates endianness.
        let flags = [e, X, X, X, X, X, X, X];        
        SubmessageHeader { 
            submessage_id: SubmessageKind::InfoDestination,
            flags,
            submessage_length: self.guid_prefix.octets() as UShort,
        }
    }
}

impl RtpsCompose for InfoDestination {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = Endianness::from(self.endianness_flag);       
        self.submessage_header().compose(writer)?;
        self.guid_prefix.serialize(writer, endianness)?;
        Ok(())
    }
}

impl RtpsParse for InfoDestination {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let guid_prefix = GuidPrefix::deserialize(&bytes[header.octets()..], endianness_flag.into())?;
        Ok(Self {endianness_flag, guid_prefix })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_info_dst_submessage_big_endian() {
        let bytes = [
            0x0E, 0b00000000, 0, 12,
            10, 11, 12, 13, // guidPrefix
            14, 15, 16, 17, // guidPrefix
            18, 19, 20, 21, // guidPrefix
        ];
        let expected = InfoDestination {
            endianness_flag: Endianness::BigEndian.into(),
            guid_prefix: GuidPrefix([10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,]),
        };
        let result = InfoDestination::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_info_dst_submessage_little_endian() {
        let bytes = [
            0x0E, 0b00000001, 12, 0,
            10, 11, 12, 13, // guidPrefix
            14, 15, 16, 17, // guidPrefix
            18, 19, 20, 21, // guidPrefix
        ];
        let expected = InfoDestination {
            endianness_flag: Endianness::LittleEndian.into(),
            guid_prefix: GuidPrefix([10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,]),
        };
        let result = InfoDestination::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

}
