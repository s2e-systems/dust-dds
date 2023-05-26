use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::submessages::InfoSourceSubmessageWrite;
use crate::implementation::rtps::messages::{
    overall_structure::SubmessageHeaderWrite, types::SubmessageKind,
};

use crate::implementation::rtps_udp_psm::mapping_traits::MappingWriteByteOrdered;

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for InfoSourceSubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite {
            submessage_id: SubmessageKind::INFO_SRC,
            flags: [
                self.endianness_flag,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
            ],
            submessage_length: 20,
        }
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        0_i32.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.protocol_version
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.vendor_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.guid_prefix
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::{
            messages::submessages::InfoSourceSubmessageRead,
            types::{GUIDPREFIX_UNKNOWN, PROTOCOLVERSION_1_0, VENDOR_ID_UNKNOWN},
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

    #[test]
    fn serialize_info_source() {
        let submessage = InfoSourceSubmessageWrite {
            endianness_flag: true,
            protocol_version: PROTOCOLVERSION_1_0,
            vendor_id: VENDOR_ID_UNKNOWN,
            guid_prefix: GUIDPREFIX_UNKNOWN,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x0c, 0b_0000_0001, 20, 0, // Submessage header
                0, 0, 0, 0, // unused
                1, 0, 0, 0, //protocol_version | vendor_id
                0, 0, 0, 0, //guid_prefix
                0, 0, 0, 0, //guid_prefix
                0, 0, 0, 0, //guid_prefix
            ]
        );
    }

    // #[test]
    // fn deserialize_info_source() {
    //     #[rustfmt::skip]
    //     let submessage = InfoSourceSubmessageRead::new(&[
    //         0x0c, 0b_0000_0001, 20, 0, // Submessage header
    //         0, 0, 0, 0, // unused
    //         1, 0, 0, 0, //protocol_version | vendor_id
    //         0, 0, 0, 0, //guid_prefix
    //         0, 0, 0, 0, //guid_prefix
    //         0, 0, 0, 0, //guid_prefix
    //     ]);

    //     let expected_endianness_flag = true;
    //     let expected_protocol_version = PROTOCOLVERSION_1_0;
    //     let expected_vendor_id = VENDOR_ID_UNKNOWN;
    //     let expected_guid_prefix = GUIDPREFIX_UNKNOWN;

    //     assert_eq!(expected_endianness_flag, submessage.endianness_flag());
    //     assert_eq!(expected_protocol_version, submessage.protocol_version());
    //     assert_eq!(expected_vendor_id, submessage.vendor_id());
    //     assert_eq!(expected_guid_prefix, submessage.guid_prefix());
    // }
}
