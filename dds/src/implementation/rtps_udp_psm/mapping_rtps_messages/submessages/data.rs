use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader,
        submessage_elements::{ParameterListSubmessageElement, SerializedDataSubmessageElement},
        submessages::DataSubmessage,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for DataSubmessage<'_> {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        let inline_qos_len = if self.inline_qos_flag {
            self.inline_qos.number_of_bytes()
        } else {
            0
        };
        let serialized_payload_len_padded = (self.serialized_payload.number_of_bytes() + 3) & !3; //ceil to multiple of 4
        let octets_to_next_header = 20 + inline_qos_len + serialized_payload_len_padded;
        RtpsSubmessageHeader {
            submessage_id: SubmessageKind::DATA,
            flags: [
                self.endianness_flag,
                self.inline_qos_flag,
                self.data_flag,
                self.key_flag,
                self.non_standard_payload_flag,
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
        const OCTETS_TO_INLINE_QOS: u16 = 16;
        const EXTRA_FLAGS: u16 = 0;
        EXTRA_FLAGS.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        OCTETS_TO_INLINE_QOS.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.reader_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.writer_sn
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        if self.inline_qos_flag {
            self.inline_qos
                .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }
        if self.data_flag || self.key_flag {
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
        }
        Ok(())
    }
}

impl<'de: 'a, 'a> MappingReadSubmessage<'de> for DataSubmessage<'a> {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        let inline_qos_flag = header.flags[1];
        let data_flag = header.flags[2];
        let key_flag = header.flags[3];
        let _extra_flags: u16 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let octets_to_inline_qos: u16 =
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let reader_id = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let writer_id = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let writer_sn = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;

        let inline_qos = if inline_qos_flag {
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?
        } else {
            ParameterListSubmessageElement { parameter: vec![] }
        };
        let inline_qos_len = if inline_qos_flag {
            inline_qos.number_of_bytes()
        } else {
            0
        };

        let serialized_payload = if data_flag || key_flag {
            let serialized_payload_length = header.submessage_length as usize
                - octets_to_inline_qos as usize
                - 4
                - inline_qos_len;
            let (data, following) = buf.split_at(serialized_payload_length as usize);
            *buf = following;
            SerializedDataSubmessageElement { value: data }
        } else {
            SerializedDataSubmessageElement { value: &[][..] }
        };

        let endianness_flag = header.flags[0];
        let non_standard_payload_flag = header.flags[4];

        Ok(Self {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            messages::submessage_elements::Parameter,
            types::{
                EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
        let serialized_payload = SerializedDataSubmessageElement { value: &[][..] };
        let submessage = DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x15_u8, 0b_0000_0001, 20, 0, // Submessage header
                0, 0, 16, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
            ]
        );
    }

    #[test]
    fn serialize_with_inline_qos_no_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let parameter_1 = Parameter {
            parameter_id: 6,
            length: 4,
            value: &[10, 11, 12, 13],
        };
        let parameter_2 = Parameter {
            parameter_id: 7,
            length: 4,
            value: &[20, 21, 22, 23],
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[][..] };

        let submessage = DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x15, 0b_0000_0011, 40, 0, // Submessage header
                0, 0, 16, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                6, 0, 4, 0, // inlineQos: parameterId_1, length_1
                10, 11, 12, 13, // inlineQos: value_1[length_1]
                7, 0, 4, 0, // inlineQos: parameterId_2, length_2
                20, 21, 22, 23, // inlineQos: value_2[length_2]
                1, 0, 0, 0, // inlineQos: Sentinel
            ]
        );
    }

    #[test]
    fn serialize_no_inline_qos_with_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = true;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
        let serialized_payload = SerializedDataSubmessageElement {
            value: &[1_u8, 2, 3, 4][..],
        };
        let submessage = DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x15, 0b_0000_0101, 24, 0, // Submessage header
                0, 0, 16, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                1, 2, 3, 4, // serialized payload
            ]
        );
    }

    #[test]
    fn serialize_no_inline_qos_with_serialized_payload_non_multiple_of_4() {
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = true;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
        let serialized_payload = SerializedDataSubmessageElement {
            value: &[1_u8, 2, 3][..],
        };
        let submessage = DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&submessage).unwrap(), vec![
                0x15, 0b_0000_0101, 24, 0, // Submessage header
                0, 0, 16, 0, // extraFlags, octetsToInlineQos
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: high
                5, 0, 0, 0, // writerSN: low
                1, 2, 3, 0, // serialized payload
            ]
        );
    }

    #[test]
    fn deserialize_no_inline_qos_no_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
        let serialized_payload = SerializedDataSubmessageElement { value: &[][..] };
        let expected = DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        };
        #[rustfmt::skip]
        let result = from_bytes(&[
            0x15, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_no_inline_qos_with_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = true;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
        let serialized_payload = SerializedDataSubmessageElement {
            value: &[1, 2, 3, 4][..],
        };
        let expected = DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        };
        #[rustfmt::skip]
        let result = from_bytes(&[
            0x15, 0b_0000_0101, 24, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            1, 2, 3, 4, // SerializedPayload
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_with_inline_qos_no_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let parameter_1 = Parameter {
            parameter_id: 6,
            length: 4,
            value: &[10, 11, 12, 13],
        };
        let parameter_2 = Parameter {
            parameter_id: 7,
            length: 4,
            value: &[20, 21, 22, 23],
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[][..] };
        let expected = DataSubmessage {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        };
        #[rustfmt::skip]
        let result = from_bytes(&[
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
