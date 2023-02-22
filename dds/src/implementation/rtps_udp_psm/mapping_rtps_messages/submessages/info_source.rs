use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::{
    overall_structure::RtpsSubmessageHeader, types::SubmessageKind,
};

use crate::implementation::rtps::messages::submessages::InfoSourceSubmessage;
use crate::implementation::rtps_udp_psm::mapping_traits::{
    MappingReadByteOrdered, MappingWriteByteOrdered,
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for InfoSourceSubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        RtpsSubmessageHeader {
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

impl<'de> MappingReadSubmessage<'de> for InfoSourceSubmessage {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        let _unused: i32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(InfoSourceSubmessage {
            endianness_flag: header.flags[0],
            protocol_version: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            vendor_id: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            guid_prefix: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::types::{GUIDPREFIX_UNKNOWN, PROTOCOLVERSION_1_0, VENDOR_ID_UNKNOWN},
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_info_source() {
        let submessage = InfoSourceSubmessage {
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

    #[test]
    fn deserialize_info_source() {
        #[rustfmt::skip]
        let buf = [
            0x0c, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 0, 0, // unused
            1, 0, 0, 0, //protocol_version | vendor_id
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
        ];

        assert_eq!(
            InfoSourceSubmessage {
                endianness_flag: true,
                protocol_version: PROTOCOLVERSION_1_0,
                vendor_id: VENDOR_ID_UNKNOWN,
                guid_prefix: GUIDPREFIX_UNKNOWN,
            },
            from_bytes(&buf).unwrap()
        );
    }
}
