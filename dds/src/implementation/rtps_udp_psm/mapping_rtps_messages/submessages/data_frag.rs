use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite, submessages::data_frag::DataFragSubmessageWrite,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for DataFragSubmessageWrite<'_> {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        // let inline_qos_len = if self.inline_qos_flag {
        //     self.inline_qos.number_of_bytes()
        // } else {
        //     0
        // };
        // let serialized_payload_len_padded = (self.serialized_payload.number_of_bytes() + 3) & !3; //ceil to multiple of 4
        // let octets_to_next_header = 32 + inline_qos_len + serialized_payload_len_padded;
        // SubmessageHeaderWrite {
        //     submessage_id: SubmessageKind::DATA_FRAG,
        //     flags: [
        //         self.endianness_flag,
        //         self.inline_qos_flag,
        //         self.key_flag,
        //         self.non_standard_payload_flag,
        //         false,
        //         false,
        //         false,
        //         false,
        //     ],
        //     submessage_length: octets_to_next_header as u16,
        // }
        todo!()
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        _writer: W,
    ) -> Result<(), Error> {
        // const OCTETS_TO_INLINE_QOS: u16 = 28;
        // const EXTRA_FLAGS: u16 = 0;
        // EXTRA_FLAGS.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // OCTETS_TO_INLINE_QOS.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.reader_id
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.writer_id
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.writer_sn
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.fragment_starting_num
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.fragments_in_submessage
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.fragment_size
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // self.data_size
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // if self.inline_qos_flag {
        //     self.inline_qos
        //         .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // }

        // self.serialized_payload
        //     .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // // Pad to 32bit boundary
        // let padding: &[u8] = match self.serialized_payload.number_of_bytes() % 4 {
        //     1 => &[0; 3],
        //     2 => &[0; 2],
        //     3 => &[0; 1],
        //     _ => &[],
        // };
        // writer.write_all(padding)?;

        // Ok(())
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            messages::{
                submessage_elements::{Parameter, ParameterList},
                types::{FragmentNumber, ParameterId, SerializedPayload, ULong, UShort},
            },
            types::{
                EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let inline_qos = &ParameterList::empty();
        let submessage = DataFragSubmessageWrite::new(
            true,
            false,
            false,
            false,
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            SequenceNumber::new(5),
            FragmentNumber::new(2),
            3,
            4,
            5,
            inline_qos,
            SerializedPayload::new(&[]),
        );
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x16_u8, 0b_0000_0001, 32, 0, // Submessage header
                0, 0, 28, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                2, 0, 0, 0, // fragmentStartingNum
                3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
                4, 0, 0, 0, // sampleSize
            ]
        );
    }

    #[test]
    fn serialize_with_inline_qos_with_serialized_payload() {
        let inline_qos = ParameterList::new(vec![Parameter::new(ParameterId(8), vec![71, 72, 73, 74])]);
        let submessage = DataFragSubmessageWrite::new(
            true,
            true,
            false,
            false,
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            SequenceNumber::new(6),
            FragmentNumber::new(2),
            3,
            8,
            5,
            &inline_qos,
            SerializedPayload::new(&[1, 2, 3]),
        );
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x16_u8, 0b_0000_0011, 48, 0, // Submessage header
                0, 0, 28, 0, // extraFlags | octetsToInlineQos
                1, 2, 3, 4, // readerId
                6, 7, 8, 9, // writerId
                0, 0, 0, 0, // writerSN: high
                6, 0, 0, 0, // writerSN: low
                2, 0, 0, 0, // fragmentStartingNum
                3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
                8, 0, 0, 0, // sampleSize
                8, 0, 4, 0, // inlineQos: parameterId, length
                71, 72, 73, 74, // inlineQos: value[length]
                1, 0, 0, 0, // inlineQos: Sentinel
                1, 2, 3, 0, // serializedPayload
            ]
        );
    }
}
