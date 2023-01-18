use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader, submessages::HeartbeatFragSubmessage,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for HeartbeatFragSubmessage {
    fn submessage_header(
        &self,
    ) -> crate::implementation::rtps::messages::overall_structure::RtpsSubmessageHeader {
        RtpsSubmessageHeader {
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

impl<'de> MappingReadSubmessage<'de> for HeartbeatFragSubmessage {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        Ok(Self {
            endianness_flag: header.flags[0],
            reader_id: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            writer_id: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            writer_sn: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            last_fragment_num: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            count: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            messages::types::FragmentNumber,
            types::{
                Count, EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_heart_beat() {
        let submessage = HeartbeatFragSubmessage {
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

    #[test]
    fn deserialize_heart_beat_frag() {
        let expected = HeartbeatFragSubmessage {
            endianness_flag: true,
            reader_id: EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            writer_id: EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            writer_sn: SequenceNumber::new(5),
            last_fragment_num: FragmentNumber::new(7),
            count: Count::new(2),
        };
        #[rustfmt::skip]
        let result = from_bytes(&[
            0x13_u8, 0b_0000_0001, 24, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: SequenceNumber: high
            5, 0, 0, 0, // writerSN: SequenceNumber: low
            7, 0, 0, 0, // lastFragmentNum
            2, 0, 0, 0, // count: Count
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
