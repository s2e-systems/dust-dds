use std::io::{Error, Write};

use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite, submessages::HeartbeatSubmessageWrite,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::MappingWriteByteOrdered,
};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for HeartbeatSubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite {
            submessage_id: SubmessageKind::HEARTBEAT,
            flags: [
                self.endianness_flag,
                self.final_flag,
                self.liveliness_flag,
                false,
                false,
                false,
                false,
                false,
            ],
            submessage_length: 28,
        }
    }

    fn mapping_write_submessage_elements<W: Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.reader_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.first_sn
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.last_sn
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.count.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::types::{
            Count, EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
            USER_DEFINED_READER_NO_KEY,
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

    #[test]
    fn serialize_heart_beat() {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = true;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let first_sn = SequenceNumber::new(5);
        let last_sn = SequenceNumber::new(7);
        let count = Count::new(2);
        let submessage = HeartbeatSubmessageWrite {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x07_u8, 0b_0000_0101, 28, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // firstSN: SequenceNumber: high
                5, 0, 0, 0, // firstSN: SequenceNumber: low
                0, 0, 0, 0, // lastSN: SequenceNumberSet: high
                7, 0, 0, 0, // lastSN: SequenceNumberSet: low
                2, 0, 0, 0, // count: Count: value (long)
            ]
        );
    }

    // #[test]
    // fn deserialize_heart_beat() {
    //     let expected_endianness_flag = true;
    //     let expected_final_flag = false;
    //     let expected_liveliness_flag = true;
    //     let expected_reader_id =
    //         EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
    //     let expected_writer_id =
    //         EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
    //     let expected_first_sn = SequenceNumber::new(5);
    //     let expected_last_sn = SequenceNumber::new(7);
    //     let expected_count = Count::new(2);
    //     #[rustfmt::skip]
    //     let submessage = HeartbeatSubmessageRead::new(&[
    //         0x07, 0b_0000_0101, 28, 0, // Submessage header
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // firstSN: SequenceNumber: high
    //         5, 0, 0, 0, // firstSN: SequenceNumber: low
    //         0, 0, 0, 0, // lastSN: SequenceNumberSet: high
    //         7, 0, 0, 0, // lastSN: SequenceNumberSet: low
    //         2, 0, 0, 0, // count: Count: value (long)
    //     ]);
    //     assert_eq!(expected_endianness_flag, submessage.endianness_flag());
    //     assert_eq!(expected_final_flag, submessage.final_flag());
    //     assert_eq!(expected_liveliness_flag, submessage.liveliness_flag());
    //     assert_eq!(expected_reader_id, submessage.reader_id());
    //     assert_eq!(expected_writer_id, submessage.writer_id());
    //     assert_eq!(expected_first_sn, submessage.first_sn());
    //     assert_eq!(expected_last_sn, submessage.last_sn());
    //     assert_eq!(expected_count, submessage.count());
    // }
}
