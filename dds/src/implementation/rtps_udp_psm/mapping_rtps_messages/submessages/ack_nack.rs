use std::io::{Error, Write};

use crate::implementation::{
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader, submessages::AckNackSubmessage,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for AckNackSubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        let octets_to_next_header = self.reader_id.number_of_bytes()
            + self.writer_id.number_of_bytes()
            + self.reader_sn_state.number_of_bytes()
            + self.count.number_of_bytes();
        RtpsSubmessageHeader {
            submessage_id: SubmessageKind::ACKNACK,
            flags: [
                self.endianness_flag,
                self.final_flag,
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

impl<'de> MappingReadSubmessage<'de> for AckNackSubmessage {
    fn mapping_read_submessage<B: byteorder::ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        let endianness_flag = header.flags[0];
        let final_flag = header.flags[1];
        let reader_id = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let writer_id = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let reader_sn_state = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let count = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;

        Ok(Self {
            endianness_flag,
            final_flag,
            reader_id,
            writer_id,
            reader_sn_state,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::messages::submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, SequenceNumberSetSubmessageElement,
        },
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_acknack() {
        let endianness_flag = true;
        let final_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: [1, 2, 3, 0x04],
        };
        let writer_id = EntityIdSubmessageElement {
            value: [6, 7, 8, 0x09],
        };
        let submessage = AckNackSubmessage {
            endianness_flag,
            final_flag,
            reader_id,
            writer_id,
            reader_sn_state: SequenceNumberSetSubmessageElement {
                base: 10,
                set: vec![],
            },
            count: CountSubmessageElement { value: 0 },
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

    #[test]
    fn deserialize_acknack() {
        #[rustfmt::skip]
        let buf = [
                0x06_u8, 0b_0000_0001, 24, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // reader_sn_state.base
               10, 0, 0, 0, // reader_sn_state.base
                0, 0, 0, 0, // reader_sn_state.set: numBits (ULong)
                0, 0, 0, 0, // count
        ];

        assert_eq!(
            AckNackSubmessage {
                endianness_flag: true,
                final_flag: false,
                reader_id: EntityIdSubmessageElement {
                    value: [1, 2, 3, 0x04],
                },
                writer_id: EntityIdSubmessageElement {
                    value: [6, 7, 8, 0x09],
                },
                reader_sn_state: SequenceNumberSetSubmessageElement {
                    base: 10,
                    set: vec![],
                },
                count: CountSubmessageElement { value: 0 },
            },
            from_bytes(&buf).unwrap()
        );
    }
}
