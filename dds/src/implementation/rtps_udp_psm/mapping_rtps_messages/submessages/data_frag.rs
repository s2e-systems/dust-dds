use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite, submessages::data_frag::DataFragSubmessageWrite,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for DataFragSubmessageWrite<'_> {
    fn submessage_header(&self) -> SubmessageHeaderWrite {
        let inline_qos_len = if self.inline_qos_flag {
            self.inline_qos.number_of_bytes()
        } else {
            0
        };
        let serialized_payload_len_padded = (self.serialized_payload.number_of_bytes() + 3) & !3; //ceil to multiple of 4
        let octets_to_next_header = 32 + inline_qos_len + serialized_payload_len_padded;
        SubmessageHeaderWrite {
            submessage_id: SubmessageKind::DATA_FRAG,
            flags: [
                self.endianness_flag,
                self.inline_qos_flag,
                self.key_flag,
                self.non_standard_payload_flag,
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
        const OCTETS_TO_INLINE_QOS: u16 = 28;
        const EXTRA_FLAGS: u16 = 0;
        EXTRA_FLAGS.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        OCTETS_TO_INLINE_QOS.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.reader_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_sn
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.fragment_starting_num
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.fragments_in_submessage
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.fragment_size
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.data_size
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        if self.inline_qos_flag {
            self.inline_qos
                .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }

        self.serialized_payload
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        // Pad to 32bit boundary
        let padding: &[u8] = match self.serialized_payload.number_of_bytes() % 4 {
            1 => &[0; 3],
            2 => &[0; 2],
            3 => &[0; 1],
            _ => &[],
        };
        writer.write_all(padding)?;

        Ok(())
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
        let submessage = DataFragSubmessageWrite {
            endianness_flag: true,
            inline_qos_flag: false,
            non_standard_payload_flag: false,
            key_flag: false,
            reader_id: EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            writer_id: EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            writer_sn: SequenceNumber::new(5),
            fragment_starting_num: FragmentNumber::new(2),
            fragments_in_submessage: UShort::new(3),
            data_size: ULong::new(4),
            fragment_size: UShort::new(5),
            inline_qos: &ParameterList::empty(),
            serialized_payload: SerializedPayload::new(&[]),
        };
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
        let submessage = DataFragSubmessageWrite {
            endianness_flag: true,
            inline_qos_flag: true,
            non_standard_payload_flag: false,
            key_flag: false,
            reader_id: EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY),
            writer_id: EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP),
            writer_sn: SequenceNumber::new(6),
            fragment_starting_num: FragmentNumber::new(2),
            fragments_in_submessage: UShort::new(3),
            data_size: ULong::new(8),
            fragment_size: UShort::new(5),
            inline_qos: &ParameterList::new(vec![Parameter::new(
                ParameterId(8),
                vec![71, 72, 73, 74],
            )]),
            serialized_payload: SerializedPayload::new(&[1, 2, 3]),
        };
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

    // #[test]
    // fn deserialize_no_inline_qos_no_serialized_payload() {
    //     #[rustfmt::skip]
    //     let submessage = DataFragSubmessageRead::new(&[
    //         0x16_u8, 0b_0000_0001, 32, 0, // Submessage header
    //         0, 0, 28, 0, // extraFlags, octetsToInlineQos
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // writerSN: high
    //         5, 0, 0, 0, // writerSN: low
    //         2, 0, 0, 0, // fragmentStartingNum
    //         3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
    //         4, 0, 0, 0, // sampleSize
    //     ]);

    //     let expected_endianness_flag = true;
    //     let expected_inline_qos_flag = false;
    //     let expected_non_standard_payload_flag = false;
    //     let expected_key_flag = false;
    //     let expected_reader_id =
    //         EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
    //     let expected_writer_id =
    //         EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
    //     let expected_writer_sn = SequenceNumber::new(5);
    //     let expected_fragment_starting_num = FragmentNumber::new(2);
    //     let expected_fragments_in_submessage = UShort::new(3);
    //     let expected_data_size = ULong::new(4);
    //     let expected_fragment_size = UShort::new(5);
    //     let expected_inline_qos = ParameterList::empty();
    //     let expected_serialized_payload = SerializedPayload::new(&[]);

    //     assert_eq!(expected_endianness_flag, submessage.endianness_flag());
    //     assert_eq!(expected_inline_qos_flag, submessage.inline_qos_flag());
    //     assert_eq!(
    //         expected_non_standard_payload_flag,
    //         submessage.non_standard_payload_flag()
    //     );
    //     assert_eq!(expected_key_flag, submessage.key_flag());
    //     assert_eq!(expected_reader_id, submessage.reader_id());
    //     assert_eq!(expected_writer_id, submessage.writer_id());
    //     assert_eq!(expected_writer_sn, submessage.writer_sn());
    //     assert_eq!(
    //         expected_fragment_starting_num,
    //         submessage.fragment_starting_num()
    //     );
    //     assert_eq!(
    //         expected_fragments_in_submessage,
    //         submessage.fragments_in_submessage()
    //     );
    //     assert_eq!(expected_data_size, submessage.data_size());
    //     assert_eq!(expected_fragment_size, submessage.fragment_size());
    //     assert_eq!(expected_inline_qos, submessage.inline_qos());
    //     assert_eq!(expected_serialized_payload, submessage.serialized_payload());
    // }

    // #[test]
    // fn deserialize_with_inline_qos_with_serialized_payload() {
    //     #[rustfmt::skip]
    //     let submessage = DataFragSubmessageRead::new(&[
    //         0x16_u8, 0b_0000_0011, 48, 0, // Submessage header
    //         0, 0, 28, 0, // extraFlags | octetsToInlineQos
    //         1, 2, 3, 4, // readerId
    //         6, 7, 8, 9, // writerId
    //         0, 0, 0, 0, // writerSN: high
    //         6, 0, 0, 0, // writerSN: low
    //         2, 0, 0, 0, // fragmentStartingNum
    //         3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
    //         8, 0, 0, 0, // sampleSize
    //         8, 0, 4, 0, // inlineQos: parameterId, length
    //         71, 72, 73, 74, // inlineQos: value[length]
    //         1, 0, 0, 0, // inlineQos: Sentinel
    //         1, 2, 3, 0, // serializedPayload
    //     ]);

    //     let expected_endianness_flag = true;
    //     let expected_inline_qos_flag = true;
    //     let expected_non_standard_payload_flag = false;
    //     let expected_key_flag = false;
    //     let expected_reader_id =
    //         EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
    //     let expected_writer_id =
    //         EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
    //     let expected_writer_sn = SequenceNumber::new(6);
    //     let expected_fragment_starting_num = FragmentNumber::new(2);
    //     let expected_fragments_in_submessage = UShort::new(3);
    //     let expected_data_size = ULong::new(8);
    //     let expected_fragment_size = UShort::new(5);
    //     let expected_inline_qos =
    //         ParameterList::new(vec![Parameter::new(ParameterId(8), vec![71, 72, 73, 74])]);
    //     let expected_serialized_payload = SerializedPayload::new(&[1, 2, 3, 0]);

    //     assert_eq!(expected_endianness_flag, submessage.endianness_flag());
    //     assert_eq!(expected_inline_qos_flag, submessage.inline_qos_flag());
    //     assert_eq!(
    //         expected_non_standard_payload_flag,
    //         submessage.non_standard_payload_flag()
    //     );
    //     assert_eq!(expected_key_flag, submessage.key_flag());
    //     assert_eq!(expected_reader_id, submessage.reader_id());
    //     assert_eq!(expected_writer_id, submessage.writer_id());
    //     assert_eq!(expected_writer_sn, submessage.writer_sn());
    //     assert_eq!(
    //         expected_fragment_starting_num,
    //         submessage.fragment_starting_num()
    //     );
    //     assert_eq!(
    //         expected_fragments_in_submessage,
    //         submessage.fragments_in_submessage()
    //     );
    //     assert_eq!(expected_data_size, submessage.data_size());
    //     assert_eq!(expected_fragment_size, submessage.fragment_size());
    //     assert_eq!(expected_inline_qos, submessage.inline_qos());
    //     assert_eq!(expected_serialized_payload, submessage.serialized_payload());
    // }
}
