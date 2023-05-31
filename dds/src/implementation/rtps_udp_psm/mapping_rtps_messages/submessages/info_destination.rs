use super::submessage::MappingWriteSubmessage;
use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite,
        submessages::info_destination::InfoDestinationSubmessageWrite, types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

impl MappingWriteSubmessage for InfoDestinationSubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        // let octets_to_next_header = self.guid_prefix.number_of_bytes();
        // SubmessageHeaderWrite {
        //     submessage_id: SubmessageKind::INFO_DST,
        //     flags: [
        //         self.endianness_flag,
        //         false,
        //         false,
        //         false,
        //         false,
        //         false,
        //         false,
        //         false,
        //     ],
        //     submessage_length: octets_to_next_header as u16,
        // }
        todo!()
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.guid_prefix
            .mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}
