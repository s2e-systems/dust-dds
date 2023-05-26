use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::submessages::NackFragSubmessageWrite;
use crate::implementation::rtps::messages::{
    overall_structure::RtpsSubmessageHeader, types::SubmessageKind,
};

use crate::implementation::rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for NackFragSubmessageWrite {
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

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::{
            messages::{
                submessage_elements::FragmentNumberSet, submessages::NackFragSubmessageRead,
                types::FragmentNumber,
            },
            types::{
                Count, EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

    #[test]
    fn serialize_nack_frag() {
        let submessage = NackFragSubmessageWrite {
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
    fn deserialize_nack_frag() {
        #[rustfmt::skip]
        let submessage = NackFragSubmessageRead::new(&[
            0x12_u8, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN
            4, 0, 0, 0, // writerSN
           10, 0, 0, 0, // fragmentNumberState.base
            0, 0, 0, 0, // fragmentNumberState.numBits
            6, 0, 0, 0, // count
        ]);

        let expected_endianness_flag = true;
        let expected_reader_id =
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let expected_writer_id =
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let expected_writer_sn = SequenceNumber::new(4);
        let expected_fragment_number_state = FragmentNumberSet {
            base: FragmentNumber::new(10),
            set: vec![],
        };
        let expected_count = Count::new(6);

        assert_eq!(expected_endianness_flag, submessage.endianness_flag());
        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_writer_sn, submessage.writer_sn());
        assert_eq!(
            expected_fragment_number_state,
            submessage.fragment_number_state()
        );
        assert_eq!(expected_count, submessage.count());
    }
}
