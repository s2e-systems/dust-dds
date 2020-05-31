use crate::types::{Ushort, LocatorList};
use super::{SubmessageKind, SubmessageFlag, SubmessageHeader, Submessage};
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, EndianessFlag, RtpsSerdesResult};

#[derive(PartialEq, Debug)]
pub struct InfoReply {
    endianness_flag: SubmessageFlag,
    multicast_flag: SubmessageFlag,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
}

impl Submessage for InfoReply {
    fn submessage_header(&self) -> SubmessageHeader {
        const X : SubmessageFlag = SubmessageFlag(false);
        let e = self.endianness_flag; 
        let m = self.multicast_flag; 
        let flags = [e, m, X, X, X, X, X, X];     
        let mut submessage_length = self.unicast_locator_list.octets();
        if self.multicast_flag.is_set() {
            submessage_length += self.multicast_locator_list.octets();
        }
        SubmessageHeader { 
            submessage_id: SubmessageKind::InfoReply,
            flags,
            submessage_length: Ushort::from(submessage_length),
        }
    }
}

impl RtpsCompose for InfoReply {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = EndianessFlag::from(self.endianness_flag);       
        self.submessage_header().compose(writer)?;
        self.unicast_locator_list.serialize(writer, endianness)?;
        if self.multicast_flag.is_set() {
            self.multicast_locator_list.serialize(writer, endianness)?;
        }
        Ok(())
    }
}

impl RtpsParse for InfoReply {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let multicast_flag = header.flags()[1];
        let unicast_locator_list = LocatorList::deserialize(&bytes[header.octets()..], endianness_flag.into())?;
        let multicast_locator_list = if multicast_flag.is_set() {
            LocatorList::deserialize(&bytes[header.octets() + unicast_locator_list.octets()..], endianness_flag.into())?
        } else {
            LocatorList(Vec::new())
        };
        Ok(Self {endianness_flag, multicast_flag, unicast_locator_list, multicast_locator_list})
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Locator};

    #[test]
    fn test_parse_info_reply_submessagewithout_multicast() {
        let bytes = [
            0x0f, 0b00000001, 0, 28,
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
            endianness_flag: EndianessFlag::LittleEndian.into(),
            multicast_flag: SubmessageFlag(false),
            unicast_locator_list: LocatorList(vec![Locator::new(100, 200, address)]),
            multicast_locator_list: LocatorList(vec![])
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
            endianness_flag: EndianessFlag::LittleEndian.into(),
            multicast_flag: SubmessageFlag(true),
            unicast_locator_list: LocatorList(vec![Locator::new(100, 200, address)]),
            multicast_locator_list: LocatorList(vec![Locator::new(101, 201, address)])
        };
        let result = InfoReply::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }



}



// #[cfg(test)]
// mod tests {
//     use super::*;

//     const SUBMESSAGE_BIG_ENDIAN: [u8; 80] = [
//         0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x10, 0x20, 0x01, 0x02, 0x03,
//         0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x00, 0x00,
//         0x00, 0x02, 0x00, 0x00, 0x11, 0x21, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01,
//         0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05,
//         0x00, 0x10, 0x15, 0x25, 0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07,
//         0x08, 0x0D, 0x0E, 0x0F, 0x10,
//     ];

//     const SUBMESSAGE_LITTLE_ENDIAN: [u8; 80] = [
//         0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x20, 0x10, 0x00, 0x00, 0x01, 0x02, 0x03,
//         0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x02, 0x00,
//         0x00, 0x00, 0x21, 0x11, 0x00, 0x00, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01,
//         0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
//         0x25, 0x15, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07,
//         0x08, 0x0D, 0x0E, 0x0F, 0x10,
//     ];

//     #[test]
//     fn test_parse_info_reply_submessage_multicast_and_unicast_big_endian() {
//         let info_reply_big_endian =
//             parse_info_reply_submessage(&SUBMESSAGE_BIG_ENDIAN, &2).unwrap();
//         assert_eq!(info_reply_big_endian.unicast_locator_list.len(), 2);

//         assert_eq!(info_reply_big_endian.unicast_locator_list[0].kind, 1);
//         assert_eq!(info_reply_big_endian.unicast_locator_list[0].port, 4128);
//         assert_eq!(
//             info_reply_big_endian.unicast_locator_list[0].address,
//             [
//                 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
//                 0x0F, 0x10,
//             ]
//         );

//         assert_eq!(info_reply_big_endian.unicast_locator_list[1].kind, 2);
//         assert_eq!(info_reply_big_endian.unicast_locator_list[1].port, 4385);
//         assert_eq!(
//             info_reply_big_endian.unicast_locator_list[1].address,
//             [
//                 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
//                 0x07, 0x08,
//             ]
//         );

//         let multicast_locator_list = info_reply_big_endian.multicast_locator_list.unwrap();
//         assert_eq!(multicast_locator_list.len(), 1);

//         assert_eq!(multicast_locator_list[0].kind, 5);
//         assert_eq!(multicast_locator_list[0].port, 1053989);
//         assert_eq!(
//             multicast_locator_list[0].address,
//             [
//                 0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07, 0x08, 0x0D, 0x0E,
//                 0x0F, 0x10,
//             ]
//         );
//     }

//     #[test]
//     fn test_parse_info_reply_submessage_unicast_big_endian() {
//         let info_reply_big_endian =
//             parse_info_reply_submessage(&SUBMESSAGE_BIG_ENDIAN, &0).unwrap();
//         assert_eq!(info_reply_big_endian.unicast_locator_list.len(), 2);

//         assert_eq!(info_reply_big_endian.unicast_locator_list[0].kind, 1);
//         assert_eq!(info_reply_big_endian.unicast_locator_list[0].port, 4128);
//         assert_eq!(
//             info_reply_big_endian.unicast_locator_list[0].address,
//             [
//                 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
//                 0x0F, 0x10,
//             ]
//         );

//         assert_eq!(info_reply_big_endian.unicast_locator_list[1].kind, 2);
//         assert_eq!(info_reply_big_endian.unicast_locator_list[1].port, 4385);
//         assert_eq!(
//             info_reply_big_endian.unicast_locator_list[1].address,
//             [
//                 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
//                 0x07, 0x08,
//             ]
//         );

//         assert!(info_reply_big_endian.multicast_locator_list.is_none());
//     }

//     #[test]
//     fn test_parse_info_reply_submessage_multicast_and_unicast_little_endian() {
//         let info_reply_little_endian =
//             parse_info_reply_submessage(&SUBMESSAGE_LITTLE_ENDIAN, &3).unwrap();
//         assert_eq!(info_reply_little_endian.unicast_locator_list.len(), 2);

//         assert_eq!(info_reply_little_endian.unicast_locator_list[0].kind, 1);
//         assert_eq!(info_reply_little_endian.unicast_locator_list[0].port, 4128);
//         assert_eq!(
//             info_reply_little_endian.unicast_locator_list[0].address,
//             [
//                 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
//                 0x0F, 0x10,
//             ]
//         );

//         assert_eq!(info_reply_little_endian.unicast_locator_list[1].kind, 2);
//         assert_eq!(info_reply_little_endian.unicast_locator_list[1].port, 4385);
//         assert_eq!(
//             info_reply_little_endian.unicast_locator_list[1].address,
//             [
//                 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
//                 0x07, 0x08,
//             ]
//         );

//         let multicast_locator_list = info_reply_little_endian.multicast_locator_list.unwrap();
//         assert_eq!(multicast_locator_list.len(), 1);

//         assert_eq!(multicast_locator_list[0].kind, 5);
//         assert_eq!(multicast_locator_list[0].port, 1053989);
//         assert_eq!(
//             multicast_locator_list[0].address,
//             [
//                 0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07, 0x08, 0x0D, 0x0E,
//                 0x0F, 0x10,
//             ]
//         );
//     }

//     #[test]
//     fn test_parse_info_reply_submessage_unicast_little_endian() {
//         let info_reply_little_endian =
//             parse_info_reply_submessage(&SUBMESSAGE_LITTLE_ENDIAN, &1).unwrap();
//         assert_eq!(info_reply_little_endian.unicast_locator_list.len(), 2);

//         assert_eq!(info_reply_little_endian.unicast_locator_list[0].kind, 1);
//         assert_eq!(info_reply_little_endian.unicast_locator_list[0].port, 4128);
//         assert_eq!(
//             info_reply_little_endian.unicast_locator_list[0].address,
//             [
//                 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
//                 0x0F, 0x10,
//             ]
//         );

//         assert_eq!(info_reply_little_endian.unicast_locator_list[1].kind, 2);
//         assert_eq!(info_reply_little_endian.unicast_locator_list[1].port, 4385);
//         assert_eq!(
//             info_reply_little_endian.unicast_locator_list[1].address,
//             [
//                 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
//                 0x07, 0x08,
//             ]
//         );

//         assert!(info_reply_little_endian.multicast_locator_list.is_none());
//     }

//     #[test]
//     fn test_parse_info_reply_submessage_unicast_and_multicast() {
//         // Test complete message with multicast and unicast
//         let submessage = [
//             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x10,
//             0x15, 0x25, 0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07, 0x08,
//             0x0D, 0x0E, 0x0F, 0x10,
//         ];

//         {
//             let info_reply = parse_info_reply_submessage(&submessage, &2).unwrap();
//             assert_eq!(info_reply.unicast_locator_list.len(), 0);

//             let multicast_locator_list = info_reply.multicast_locator_list.unwrap();
//             assert_eq!(multicast_locator_list.len(), 1);

//             assert_eq!(multicast_locator_list[0].kind, 5);
//             assert_eq!(multicast_locator_list[0].port, 1053989);
//             assert_eq!(
//                 multicast_locator_list[0].address,
//                 [
//                     0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07, 0x08, 0x0D,
//                     0x0E, 0x0F, 0x10,
//                 ]
//             );
//         }

//         {
//             let info_reply = parse_info_reply_submessage(&submessage, &0).unwrap();
//             assert_eq!(info_reply.unicast_locator_list.len(), 0);

//             assert!(info_reply.multicast_locator_list.is_none());
//         }
//     }
// }
