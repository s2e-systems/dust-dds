use std::io::{Error, Write};

use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite,
        types::SubmessageKind, submessages::ack_nack::AckNackSubmessageWrite,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
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
        self.reader_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.reader_sn_state
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.count.mapping_write_byte_ordered::<_, B>(&mut writer)?;

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

    #[test]
    fn serialize_acknack() {
        let endianness_flag = true;
        let final_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let submessage = AckNackSubmessageWrite {
            endianness_flag,
            final_flag,
            reader_id,
            writer_id,
            reader_sn_state: SequenceNumberSet {
                base: SequenceNumber::new(10),
                set: vec![],
            },
            count: Count::new(0),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x06_u8, 0b_0000_0001, 24, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // reader_sn_state.base
               10, 0, 0, 0, // reader_sn_state.base
                0, 0, 0, 0, // reader_sn_state.set: numBits (ULong)
                0, 0, 0, 0, // count
            ]
        );
    }

}
