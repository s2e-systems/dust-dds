use crate::types::LocatorList;

use super::helpers::{deserialize, endianess};

use super::Result;

#[derive(PartialEq, Debug)]
pub struct InfoReply {
    unicast_locator_list: LocatorList,
    multicast_locator_list: Option<LocatorList>,
}

pub fn parse_info_reply_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<InfoReply> {
    const MULTICAST_FLAG_MASK: u8 = 0x02;
    const UNICAST_LOCATOR_LIST_FIRST_INDEX: usize = 0;
    const UNICAST_LOCATOR_LIST_MINIMUM_SIZE: usize = 4;
    const UNICAST_LOCATOR_SIZE: usize = 24;

    let submessage_last_index = submessage.len() - 1;

    let submessage_endianess = endianess(submessage_flags)?;
    let multicast_flag = submessage_flags & MULTICAST_FLAG_MASK == MULTICAST_FLAG_MASK;

    let unicast_locator_list = deserialize::<LocatorList>(
        submessage,
        &UNICAST_LOCATOR_LIST_FIRST_INDEX,
        &submessage_last_index,
        &submessage_endianess,
    )?;

    let multicast_locator_list = if multicast_flag {
        let multicast_locator_list_first_index = UNICAST_LOCATOR_LIST_FIRST_INDEX
            + (UNICAST_LOCATOR_SIZE * unicast_locator_list.len())
            + UNICAST_LOCATOR_LIST_MINIMUM_SIZE;
        Some(deserialize::<LocatorList>(
            submessage,
            &multicast_locator_list_first_index,
            &submessage_last_index,
            &submessage_endianess,
        )?)
    } else {
        None
    };

    Ok(InfoReply {
        unicast_locator_list,
        multicast_locator_list,
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    const SUBMESSAGE_BIG_ENDIAN: [u8; 80] = [
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x10, 0x20, 0x01, 0x02, 0x03,
        0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x00, 0x00,
        0x00, 0x02, 0x00, 0x00, 0x11, 0x21, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01,
        0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05,
        0x00, 0x10, 0x15, 0x25, 0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07,
        0x08, 0x0D, 0x0E, 0x0F, 0x10,
    ];

    const SUBMESSAGE_LITTLE_ENDIAN: [u8; 80] = [
        0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x20, 0x10, 0x00, 0x00, 0x01, 0x02, 0x03,
        0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x02, 0x00,
        0x00, 0x00, 0x21, 0x11, 0x00, 0x00, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01,
        0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
        0x25, 0x15, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07,
        0x08, 0x0D, 0x0E, 0x0F, 0x10,
    ];

    #[test]
    fn test_parse_info_reply_submessage_multicast_and_unicast_big_endian() {
        let info_reply_big_endian =
            parse_info_reply_submessage(&SUBMESSAGE_BIG_ENDIAN, &2).unwrap();
        assert_eq!(info_reply_big_endian.unicast_locator_list.len(), 2);

        assert_eq!(info_reply_big_endian.unicast_locator_list[0].kind, 1);
        assert_eq!(info_reply_big_endian.unicast_locator_list[0].port, 4128);
        assert_eq!(
            info_reply_big_endian.unicast_locator_list[0].address,
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                0x0F, 0x10,
            ]
        );

        assert_eq!(info_reply_big_endian.unicast_locator_list[1].kind, 2);
        assert_eq!(info_reply_big_endian.unicast_locator_list[1].port, 4385);
        assert_eq!(
            info_reply_big_endian.unicast_locator_list[1].address,
            [
                0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08,
            ]
        );

        let multicast_locator_list = info_reply_big_endian.multicast_locator_list.unwrap();
        assert_eq!(multicast_locator_list.len(), 1);

        assert_eq!(multicast_locator_list[0].kind, 5);
        assert_eq!(multicast_locator_list[0].port, 1053989);
        assert_eq!(
            multicast_locator_list[0].address,
            [
                0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07, 0x08, 0x0D, 0x0E,
                0x0F, 0x10,
            ]
        );
    }

    #[test]
    fn test_parse_info_reply_submessage_unicast_big_endian() {
        let info_reply_big_endian =
            parse_info_reply_submessage(&SUBMESSAGE_BIG_ENDIAN, &0).unwrap();
        assert_eq!(info_reply_big_endian.unicast_locator_list.len(), 2);

        assert_eq!(info_reply_big_endian.unicast_locator_list[0].kind, 1);
        assert_eq!(info_reply_big_endian.unicast_locator_list[0].port, 4128);
        assert_eq!(
            info_reply_big_endian.unicast_locator_list[0].address,
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                0x0F, 0x10,
            ]
        );

        assert_eq!(info_reply_big_endian.unicast_locator_list[1].kind, 2);
        assert_eq!(info_reply_big_endian.unicast_locator_list[1].port, 4385);
        assert_eq!(
            info_reply_big_endian.unicast_locator_list[1].address,
            [
                0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08,
            ]
        );

        assert!(info_reply_big_endian.multicast_locator_list.is_none());
    }

    #[test]
    fn test_parse_info_reply_submessage_multicast_and_unicast_little_endian() {
        let info_reply_little_endian =
            parse_info_reply_submessage(&SUBMESSAGE_LITTLE_ENDIAN, &3).unwrap();
        assert_eq!(info_reply_little_endian.unicast_locator_list.len(), 2);

        assert_eq!(info_reply_little_endian.unicast_locator_list[0].kind, 1);
        assert_eq!(info_reply_little_endian.unicast_locator_list[0].port, 4128);
        assert_eq!(
            info_reply_little_endian.unicast_locator_list[0].address,
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                0x0F, 0x10,
            ]
        );

        assert_eq!(info_reply_little_endian.unicast_locator_list[1].kind, 2);
        assert_eq!(info_reply_little_endian.unicast_locator_list[1].port, 4385);
        assert_eq!(
            info_reply_little_endian.unicast_locator_list[1].address,
            [
                0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08,
            ]
        );

        let multicast_locator_list = info_reply_little_endian.multicast_locator_list.unwrap();
        assert_eq!(multicast_locator_list.len(), 1);

        assert_eq!(multicast_locator_list[0].kind, 5);
        assert_eq!(multicast_locator_list[0].port, 1053989);
        assert_eq!(
            multicast_locator_list[0].address,
            [
                0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07, 0x08, 0x0D, 0x0E,
                0x0F, 0x10,
            ]
        );
    }

    #[test]
    fn test_parse_info_reply_submessage_unicast_little_endian() {
        let info_reply_little_endian =
            parse_info_reply_submessage(&SUBMESSAGE_LITTLE_ENDIAN, &1).unwrap();
        assert_eq!(info_reply_little_endian.unicast_locator_list.len(), 2);

        assert_eq!(info_reply_little_endian.unicast_locator_list[0].kind, 1);
        assert_eq!(info_reply_little_endian.unicast_locator_list[0].port, 4128);
        assert_eq!(
            info_reply_little_endian.unicast_locator_list[0].address,
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                0x0F, 0x10,
            ]
        );

        assert_eq!(info_reply_little_endian.unicast_locator_list[1].kind, 2);
        assert_eq!(info_reply_little_endian.unicast_locator_list[1].port, 4385);
        assert_eq!(
            info_reply_little_endian.unicast_locator_list[1].address,
            [
                0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
                0x07, 0x08,
            ]
        );

        assert!(info_reply_little_endian.multicast_locator_list.is_none());
    }

    #[test]
    fn test_parse_info_reply_submessage_unicast_and_multicast() {
        // Test complete message with multicast and unicast
        let submessage = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x10,
            0x15, 0x25, 0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07, 0x08,
            0x0D, 0x0E, 0x0F, 0x10,
        ];

        {
            let info_reply = parse_info_reply_submessage(&submessage, &2).unwrap();
            assert_eq!(info_reply.unicast_locator_list.len(), 0);

            let multicast_locator_list = info_reply.multicast_locator_list.unwrap();
            assert_eq!(multicast_locator_list.len(), 1);

            assert_eq!(multicast_locator_list[0].kind, 5);
            assert_eq!(multicast_locator_list[0].port, 1053989);
            assert_eq!(
                multicast_locator_list[0].address,
                [
                    0x01, 0x02, 0x03, 0x04, 0x09, 0x0A, 0x0B, 0x0C, 0x05, 0x06, 0x07, 0x08, 0x0D,
                    0x0E, 0x0F, 0x10,
                ]
            );
        }

        {
            let info_reply = parse_info_reply_submessage(&submessage, &0).unwrap();
            assert_eq!(info_reply.unicast_locator_list.len(), 0);

            assert!(info_reply.multicast_locator_list.is_none());
        }
    }
}
