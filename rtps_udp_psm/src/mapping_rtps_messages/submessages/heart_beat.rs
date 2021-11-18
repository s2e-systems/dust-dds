use rust_rtps_pim::messages::{overall_structure::RtpsSubmessageHeader, types::SubmessageKind};
use rust_rtps_psm::messages::submessages::{HeartbeatSubmessageRead, HeartbeatSubmessageWrite};

use crate::{
    deserialize::{self, MappingReadByteOrdered},
    serialize::MappingWriteByteOrdered,
};

use std::io::Write;

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for HeartbeatSubmessageWrite {
    fn submessage_header(
        &self,
    ) -> rust_rtps_pim::messages::overall_structure::RtpsSubmessageHeader {
        RtpsSubmessageHeader {
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
    ) -> crate::serialize::Result {
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

impl<'de> MappingReadSubmessage<'de> for HeartbeatSubmessageRead {
    fn mapping_read_submessage<B: byteorder::ByteOrder>(
        buf: &mut &'de [u8],
        header: rust_rtps_pim::messages::overall_structure::RtpsSubmessageHeader,
    ) -> deserialize::Result<Self> {
        let reader_id = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let writer_id = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let first_sn = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let last_sn = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let count = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(Self::new(
            header.flags[0],
            header.flags[1],
            header.flags[2],
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{deserialize::from_bytes, serialize::to_bytes};

    use super::*;
    use rust_rtps_pim::{
        messages::{
            submessage_elements::{
                CountSubmessageElement, EntityIdSubmessageElement, SequenceNumberSubmessageElement,
            },
            types::Count,
        },
        structure::types::{EntityId, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };
    #[test]
    fn serialize_heart_beat() {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = true;
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
        };
        let first_sn = SequenceNumberSubmessageElement { value: 5 };
        let last_sn = SequenceNumberSubmessageElement { value: 7 };
        let count = CountSubmessageElement { value: Count(2) };
        let submessage = HeartbeatSubmessageWrite::new(
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        );
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

    #[test]
    fn deserialize_heart_beat() {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = true;
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
        };
        let first_sn = SequenceNumberSubmessageElement { value: 5 };
        let last_sn = SequenceNumberSubmessageElement { value: 7 };
        let count = CountSubmessageElement { value: Count(2) };
        let expected = HeartbeatSubmessageRead::new(
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        );
        #[rustfmt::skip]
        let result = from_bytes(&[
            0x07, 0b_0000_0101, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
