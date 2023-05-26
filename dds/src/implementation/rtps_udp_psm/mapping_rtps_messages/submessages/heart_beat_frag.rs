use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite, submessages::HeartbeatFragSubmessageWrite,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::MappingWriteByteOrdered,
};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for HeartbeatFragSubmessageWrite {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite {
            submessage_id: SubmessageKind::HEARTBEAT_FRAG,
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
            submessage_length: 24,
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
        self.writer_sn
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.last_fragment_num
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.count.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            messages::{submessages::HeartbeatFragSubmessageRead, types::FragmentNumber},
            types::{
                Count, EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

    #[test]
    fn serialize_heart_beat() {
        let submessage = HeartbeatFragSubmessageWrite {
            endianness_flag: true,
            reader_id: EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            writer_id: EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            writer_sn: SequenceNumber::new(5),
            last_fragment_num: FragmentNumber::new(7),
            count: Count::new(2),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x13_u8, 0b_0000_0001, 24, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: SequenceNumber: high
                5, 0, 0, 0, // writerSN: SequenceNumber: low
                7, 0, 0, 0, // lastFragmentNum
                2, 0, 0, 0, // count: Count
            ]
        );
    }

    // #[test]
    // fn deserialize_heart_beat_frag() {
    //     #[rustfmt::skip]
    //     let submessage = HeartbeatFragSubmessageRead::new(&[
    //         0x13_u8, 0b_0000_0001, 24, 0, // Submessage header
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // writerSN: SequenceNumber: high
    //         5, 0, 0, 0, // writerSN: SequenceNumber: low
    //         7, 0, 0, 0, // lastFragmentNum
    //         2, 0, 0, 0, // count: Count
    //     ]);

    //     let expected_endianness_flag = true;
    //     let expected_reader_id =
    //         EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
    //     let expected_writer_id =
    //         EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
    //     let expected_writer_sn = SequenceNumber::new(5);
    //     let expected_last_fragment_num = FragmentNumber::new(7);
    //     let expected_count = Count::new(2);

    //     assert_eq!(expected_endianness_flag, submessage.endianness_flag());
    //     assert_eq!(expected_reader_id, submessage.reader_id());
    //     assert_eq!(expected_writer_id, submessage.writer_id());
    //     assert_eq!(expected_writer_sn, submessage.writer_sn());
    //     assert_eq!(expected_last_fragment_num, submessage.last_fragment_num());
    //     assert_eq!(expected_count, submessage.count());
    // }
}
