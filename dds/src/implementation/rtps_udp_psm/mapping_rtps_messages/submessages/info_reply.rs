use super::submessage::MappingWriteSubmessage;
use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite,
        submessages::info_reply::InfoReplySubmessageWrite, types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

impl MappingWriteSubmessage for InfoReplySubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        // let unicast_locator_list_number_of_bytes = self.unicast_locator_list.number_of_bytes();
        // let submessage_length = match self.multicast_flag {
        //     true => {
        //         unicast_locator_list_number_of_bytes + self.multicast_locator_list.number_of_bytes()
        //     }
        //     false => unicast_locator_list_number_of_bytes,
        // } as u16;
        // SubmessageHeaderWrite {
        //     submessage_id: SubmessageKind::INFO_REPLY,
        //     flags: [
        //         self.endianness_flag,
        //         self.multicast_flag,
        //         false,
        //         false,
        //         false,
        //         false,
        //         false,
        //         false,
        //     ],
        //     submessage_length,
        // }
        todo!()
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

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::{
            messages::submessage_elements::LocatorList,
            types::{Locator, LocatorAddress, LocatorKind, LocatorPort},
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

    #[test]
    fn serialize_info_reply() {
        let locator = Locator::new(
            LocatorKind::new(11),
            LocatorPort::new(12),
            LocatorAddress::new([1; 16]),
        );
        let submessage = InfoReplySubmessageWrite {
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
        let submessage = InfoReplySubmessageWrite {
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

}
