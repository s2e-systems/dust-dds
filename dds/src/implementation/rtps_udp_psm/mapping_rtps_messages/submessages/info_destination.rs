use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader, submessages::InfoDestinationSubmessage,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for InfoDestinationSubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        let octets_to_next_header = self.guid_prefix.number_of_bytes();
        RtpsSubmessageHeader {
            submessage_id: SubmessageKind::INFO_DST,
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
            submessage_length: octets_to_next_header as u16,
        }
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.guid_prefix
            .mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadSubmessage<'de> for InfoDestinationSubmessage {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        let endianness_flag = header.flags[0];
        let guid_prefix = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(Self {
            endianness_flag,
            guid_prefix,
        })
    }
}
