use crate::serdes::{SubmessageElement, Endianness, RtpsSerdesResult, };

use super::types::{SubmessageKind, SubmessageFlag, };
use super::{SubmessageHeader, Submessage, };
use super::submessage_elements;

#[derive(PartialEq, Debug)]
pub struct InfoReply {
    endianness_flag: SubmessageFlag,
    multicast_flag: SubmessageFlag,
    unicast_locator_list: submessage_elements::LocatorList,
    multicast_locator_list: submessage_elements::LocatorList,
}

impl Submessage for InfoReply {
    fn submessage_header(&self) -> SubmessageHeader {
        const X : SubmessageFlag = false;
        let e = self.endianness_flag; 
        let m = self.multicast_flag; 
        let flags = [e, m, X, X, X, X, X, X];     
        let mut submessage_length = self.unicast_locator_list.octets();
        if self.multicast_flag {
            submessage_length += self.multicast_locator_list.octets();
        }
        SubmessageHeader { 
            submessage_id: SubmessageKind::InfoReply,
            flags,
            submessage_length: submessage_length as u16,
        }
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = Endianness::from(self.endianness_flag);       
        self.submessage_header().compose(writer)?;
        self.unicast_locator_list.serialize(writer, endianness)?;
        if self.multicast_flag {
            self.multicast_locator_list.serialize(writer, endianness)?;
        }
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let multicast_flag = header.flags()[1];
        let unicast_locator_list = submessage_elements::LocatorList::deserialize(&bytes[header.octets()..], endianness_flag.into())?;
        let multicast_locator_list = if multicast_flag {
            submessage_elements::LocatorList::deserialize(&bytes[header.octets() + unicast_locator_list.octets()..], endianness_flag.into())?
        } else {
            submessage_elements::LocatorList(Vec::new())
        };
        Ok(Self {endianness_flag, multicast_flag, unicast_locator_list, multicast_locator_list})
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Locator;

    #[test]
    fn test_parse_info_reply_submessage_without_multicast() {
        let bytes = [
            0x0f, 0b00000001, 28, 0, 
            1, 0, 0, 0,   // numLocators
            100, 0, 0, 0, // Locator 1: kind
            200, 0, 0, 0, // Locator 1: port
             1,  2,  3,  4, // Locator 1: address
             5,  6,  7,  8, // Locator 1: address
             9, 10, 11, 12, // Locator 1: address
            13, 14, 15, 16, // Locator 1: address
        ];
        let address = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let expected = InfoReply {
            endianness_flag: Endianness::LittleEndian.into(),
            multicast_flag: false,
            unicast_locator_list: submessage_elements::LocatorList(vec![Locator::new(100, 200, address)]),
            multicast_locator_list: submessage_elements::LocatorList(vec![])
        };
        let result = InfoReply::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_info_reply_submessage_with_multicast() {
        let bytes = [
            0x0f, 0b00000011, 0, 56,
            1, 0, 0, 0,   // numLocators
            100, 0, 0, 0, // Locator 1: kind
            200, 0, 0, 0, // Locator 1: port
             1,  2,  3,  4, // Locator 1: address
             5,  6,  7,  8, // Locator 1: address
             9, 10, 11, 12, // Locator 1: address
            13, 14, 15, 16, // Locator 1: address
            1, 0, 0, 0,   // numLocators
            101, 0, 0, 0, // Locator 1: kind
            201, 0, 0, 0, // Locator 1: port
             1,  2,  3,  4, // Locator 1: address
             5,  6,  7,  8, // Locator 1: address
             9, 10, 11, 12, // Locator 1: address
            13, 14, 15, 16, // Locator 1: address
        ];
        let address = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let expected = InfoReply {
            endianness_flag: Endianness::LittleEndian.into(),
            multicast_flag: true,
            unicast_locator_list: submessage_elements::LocatorList(vec![Locator::new(100, 200, address)]),
            multicast_locator_list: submessage_elements::LocatorList(vec![Locator::new(101, 201, address)])
        };
        let result = InfoReply::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_compose_info_reply_submessage_with_multicast() {
        let expected = vec![
            0x0f, 0b00000011, 56, 0, 
            1, 0, 0, 0,   // numLocators
            100, 0, 0, 0, // Locator 1: kind
            200, 0, 0, 0, // Locator 1: port
             1,  2,  3,  4, // Locator 1: address
             5,  6,  7,  8, // Locator 1: address
             9, 10, 11, 12, // Locator 1: address
            13, 14, 15, 16, // Locator 1: address
            1, 0, 0, 0,   // numLocators
            101, 0, 0, 0, // Locator 1: kind
            201, 0, 0, 0, // Locator 1: port
             1,  2,  3,  4, // Locator 1: address
             5,  6,  7,  8, // Locator 1: address
             9, 10, 11, 12, // Locator 1: address
            13, 14, 15, 16, // Locator 1: address
        ];
        let address = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let message = InfoReply {
            endianness_flag: Endianness::LittleEndian.into(),
            multicast_flag: true,
            unicast_locator_list: submessage_elements::LocatorList(vec![Locator::new(100, 200, address)]),
            multicast_locator_list: submessage_elements::LocatorList(vec![Locator::new(101, 201, address)])
        };
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }

}
