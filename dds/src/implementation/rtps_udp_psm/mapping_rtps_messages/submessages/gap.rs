use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite, submessages::GapSubmessageWrite,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for GapSubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        let submessage_length = 16 + self.gap_list.number_of_bytes();
        SubmessageHeaderWrite {
            submessage_id: SubmessageKind::GAP,
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
            submessage_length: submessage_length as u16,
        }
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.reader_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.gap_start
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.gap_list
            .mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            messages::submessage_elements::SequenceNumberSet,
            types::{
                EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

    #[test]
    fn serialize_gap() {
        let endianness_flag = true;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let gap_start = SequenceNumber::new(5);
        let gap_list = SequenceNumberSet {
            base: SequenceNumber::new(10),
            set: vec![],
        };
        let submessage = GapSubmessageWrite {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x08_u8, 0b_0000_0001, 28, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // gapStart: SequenceNumber: high
                5, 0, 0, 0, // gapStart: SequenceNumber: low
                0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
               10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
                0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
            ]
        );
    }

    // #[test]
    // fn deserialize_gap() {
    //     let expected_endianness_flag = true;
    //     let expected_reader_id =
    //         EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
    //     let expected_writer_id =
    //         EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
    //     let expected_gap_start = SequenceNumber::new(5);
    //     let expected_gap_list = SequenceNumberSet {
    //         base: SequenceNumber::new(10),
    //         set: vec![],
    //     };
    //     #[rustfmt::skip]
    //     let submessage = GapSubmessageRead::new(&[
    //         0x08, 0b_0000_0001, 28, 0, // Submessage header
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // gapStart: SequenceNumber: high
    //         5, 0, 0, 0, // gapStart: SequenceNumber: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
    //        10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
    //     ]);
    //     assert_eq!(expected_endianness_flag, submessage.endianness_flag());
    //     assert_eq!(expected_reader_id, submessage.reader_id());
    //     assert_eq!(expected_writer_id, submessage.writer_id());
    //     assert_eq!(expected_gap_start, submessage.gap_start());
    //     assert_eq!(expected_gap_list, submessage.gap_list());
    // }
}
