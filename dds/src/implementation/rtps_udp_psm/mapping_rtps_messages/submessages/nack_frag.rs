use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::{
    overall_structure::RtpsSubmessageHeader, types::SubmessageKind,
};

use crate::implementation::rtps::messages::submessages::NackFragSubmessage;
use crate::implementation::rtps_udp_psm::mapping_traits::{NumberOfBytes, MappingWriteByteOrdered, MappingReadByteOrdered};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for NackFragSubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        let octets_to_next_header = self.reader_id.number_of_bytes()
            + self.writer_id.number_of_bytes()
            + self.writer_sn.number_of_bytes()
            + self.fragment_number_state.number_of_bytes()
            + self.count.number_of_bytes();
        RtpsSubmessageHeader {
            submessage_id: SubmessageKind::NACK_FRAG,
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
        self.reader_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_sn
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.fragment_number_state
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.count.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        Ok(())
    }
}

impl<'de> MappingReadSubmessage<'de> for NackFragSubmessage {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
         Ok(Self {
            endianness_flag: header.flags[0],
            reader_id: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            writer_id: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            writer_sn: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            fragment_number_state: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            count: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::{
            messages::{submessage_elements::FragmentNumberSet, types::FragmentNumber},
            types::{
                Count, EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_nack_frag() {
        let submessage = NackFragSubmessage {
            endianness_flag: true,
            reader_id: EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            writer_id: EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            writer_sn: SequenceNumber::new(4),
            fragment_number_state: FragmentNumberSet {
                base: FragmentNumber::new(10),
                set: vec![],
            },
            count: Count::new(6),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x12_u8, 0b_0000_0001, 28, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN
                4, 0, 0, 0, // writerSN
               10, 0, 0, 0, // fragmentNumberState.base
                0, 0, 0, 0, // fragmentNumberState.numBits
                6, 0, 0, 0, // count
            ]
        );
    }

    #[test]
    fn deserialize_acknack() {
        #[rustfmt::skip]
        let buf = [
            0x12_u8, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN
            4, 0, 0, 0, // writerSN
           10, 0, 0, 0, // fragmentNumberState.base
            0, 0, 0, 0, // fragmentNumberState.numBits
            6, 0, 0, 0, // count
        ];

        assert_eq!(
            NackFragSubmessage {
                endianness_flag: true,
                reader_id: EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
                writer_id: EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
                writer_sn: SequenceNumber::new(4),
                fragment_number_state: FragmentNumberSet {
                    base: FragmentNumber::new(10),
                    set: vec![],
                },
                count: Count::new(6),
            },
            from_bytes(&buf).unwrap()
        );
    }
}
