use byteorder::LittleEndian;
use rtps_pim::{
    messages::{
        overall_structure::RtpsSubmessageHeader, submessages::AckNackSubmessage,
        types::SubmessageKind,
    },
    structure::types::SequenceNumber,
};

use crate::mapping_traits::{MappingRead, MappingWriteByteOrdered, NumberOfBytes};

use std::io::{Error, Write};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for AckNackSubmessage<Vec<SequenceNumber>> {
    fn submessage_header(&self) -> rtps_pim::messages::overall_structure::RtpsSubmessageHeader {
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
            .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
        self.writer_id
            .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
        self.reader_sn_state
            .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
        self.count
            .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;

        Ok(())
    }
}

impl<'de, S> MappingRead<'de> for AckNackSubmessage<S> {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::{
        messages::{
            submessage_elements::{
                CountSubmessageElement, EntityIdSubmessageElement,
                SequenceNumberSetSubmessageElement,
            },
            submessages::AckNackSubmessage,
            types::Count,
        },
        structure::types::{EntityId, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    use crate::mapping_traits::to_bytes;

    #[test]
    fn acknack() {
        let endianness_flag = true;
        let final_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
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
            count: CountSubmessageElement { value: Count(0) },
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
