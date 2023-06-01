use std::io::{Error, Write};

use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite,
        submessages::ack_nack::AckNackSubmessageWrite,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, },
};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for AckNackSubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        // let octets_to_next_header = self.reader_id.number_of_bytes()
        //     + self.writer_id.number_of_bytes()
        //     + self.reader_sn_state.number_of_bytes()
        //     + self.count.number_of_bytes();
        // SubmessageHeaderWrite {
        //     submessage_id: SubmessageKind::ACKNACK,
        //     flags: [
        //         self.endianness_flag,
        //         self.final_flag,
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

    fn mapping_write_submessage_elements<W: Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        // self.reader_id
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.writer_id
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.reader_sn_state
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.count.mapping_write_byte_ordered::<_, B>(&mut writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::{
            messages::submessage_elements::SequenceNumberSet,
            types::{
                Count, EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

}
