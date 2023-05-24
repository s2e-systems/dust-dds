use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader, submessage_elements::LocatorList,
        submessages::InfoReplySubmessage, types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for InfoReplySubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        let unicast_locator_list_number_of_bytes = self.unicast_locator_list.number_of_bytes();
        let submessage_length = match self.multicast_flag {
            true => {
                unicast_locator_list_number_of_bytes + self.multicast_locator_list.number_of_bytes()
            }
            false => unicast_locator_list_number_of_bytes,
        } as u16;
        RtpsSubmessageHeader {
            submessage_id: SubmessageKind::INFO_REPLY,
            flags: [
                self.endianness_flag,
                self.multicast_flag,
                false,
                false,
                false,
                false,
                false,
                false,
            ],
            submessage_length,
        }
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.unicast_locator_list
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        if self.multicast_flag {
            self.multicast_locator_list
                .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }
        Ok(())
    }
}

impl<'de> MappingReadSubmessage<'de> for InfoReplySubmessage {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        let endianness_flag = header.flags[0];
        let multicast_flag = header.flags[1];
        let unicast_locator_list = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let multicast_locator_list = if multicast_flag {
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?
        } else {
            LocatorList::new(vec![])
        };
        Ok(InfoReplySubmessage {
            endianness_flag,
            multicast_flag,
            unicast_locator_list,
            multicast_locator_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::{
            messages::submessage_elements::LocatorList,
            types::{Locator, LocatorAddress, LocatorKind, LocatorPort},
        },
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_info_reply() {
        let locator = Locator::new(
            LocatorKind::new(11),
            LocatorPort::new(12),
            LocatorAddress::new([1; 16]),
        );
        let submessage = InfoReplySubmessage {
            endianness_flag: true,
            multicast_flag: false,
            unicast_locator_list: LocatorList::new(vec![locator]),
            multicast_locator_list: LocatorList::new(vec![]),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x0f, 0b_0000_0001, 28, 0, // Submessage header
                1, 0, 0, 0, //numLocators
                11, 0, 0, 0, //kind
                12, 0, 0, 0, //port
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
            ]
        );
    }

    #[test]
    fn serialize_info_reply_with_multicast() {
        let locator = Locator::new(
            LocatorKind::new(11),
            LocatorPort::new(12),
            LocatorAddress::new([1; 16]),
        );
        let submessage = InfoReplySubmessage {
            endianness_flag: true,
            multicast_flag: true,
            unicast_locator_list: LocatorList::new(vec![]),
            multicast_locator_list: LocatorList::new(vec![locator, locator]),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x0f, 0b_0000_0011, 56, 0, // Submessage header
                0, 0, 0, 0, //numLocators
                2, 0, 0, 0, //numLocators
                11, 0, 0, 0, //kind
                12, 0, 0, 0, //port
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                11, 0, 0, 0, //kind
                12, 0, 0, 0, //port
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
            ]
        );
    }

    #[test]
    fn deserialize_info_reply() {
        #[rustfmt::skip]
        let buf = [
            0x0f, 0b_0000_0001, 28, 0, // Submessage header
            1, 0, 0, 0, //numLocators
            11, 0, 0, 0, //kind
            12, 0, 0, 0, //port
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
        ];

        let locator = Locator::new(
            LocatorKind::new(11),
            LocatorPort::new(12),
            LocatorAddress::new([1; 16]),
        );
        assert_eq!(
            InfoReplySubmessage {
                endianness_flag: true,
                multicast_flag: false,
                unicast_locator_list: LocatorList::new(vec![locator]),
                multicast_locator_list: LocatorList::new(vec![]),
            },
            from_bytes(&buf).unwrap()
        );
    }

    #[test]
    fn deserialize_info_reply_with_multicast() {
        #[rustfmt::skip]
        let buf = [
            0x0f, 0b_0000_0011, 56, 0, // Submessage header
            0, 0, 0, 0, //numLocators
            2, 0, 0, 0, //numLocators
            11, 0, 0, 0, //kind
            12, 0, 0, 0, //port
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            11, 0, 0, 0, //kind
            12, 0, 0, 0, //port
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
        ];

        let locator = Locator::new(
            LocatorKind::new(11),
            LocatorPort::new(12),
            LocatorAddress::new([1; 16]),
        );
        assert_eq!(
            InfoReplySubmessage {
                endianness_flag: true,
                multicast_flag: true,
                unicast_locator_list: LocatorList::new(vec![]),
                multicast_locator_list: LocatorList::new(vec![locator, locator]),
            },
            from_bytes(&buf).unwrap()
        );
    }
}
