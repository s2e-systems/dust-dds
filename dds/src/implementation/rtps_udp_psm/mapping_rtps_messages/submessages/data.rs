use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader,
        submessages::{DataSubmessageRead, DataSubmessageWrite},
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for DataSubmessageWrite<'_> {
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

impl<'de: 'a, 'a> MappingReadSubmessage<'de> for DataSubmessageRead<'a> {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        let (data, following) = buf.split_at(header.submessage_length as usize);
        *buf = following;

        Ok(Self::new(data))
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            messages::{
                submessage_elements::{Parameter, ParameterList},
                types::{ParameterId, SerializedPayload},
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
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let inline_qos = &ParameterList::empty();
        let serialized_payload = SerializedPayload::new(&[]);
        let submessage = DataSubmessageWrite {
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
        let parameter_1 = Parameter::new(ParameterId(6), vec![10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), vec![20, 21, 22, 23]);
        let inline_qos = &ParameterList::new(vec![parameter_1, parameter_2]);
        let serialized_payload = SerializedPayload::new(&[]);

        let submessage = DataSubmessageWrite {
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
        let inline_qos = &ParameterList::empty();
        let serialized_payload = SerializedPayload::new(&[1, 2, 3, 4]);
        let submessage = DataSubmessageWrite {
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
        let inline_qos = &ParameterList::empty();
        let serialized_payload = SerializedPayload::new(&[1, 2, 3]);
        let submessage = DataSubmessageWrite {
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
        let inline_qos = ParameterList::empty();
        let serialized_payload = SerializedPayload::new(&[]);

        #[rustfmt::skip]
        let data_submessage = DataSubmessageRead::new(&[
            0x15, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
        ]);

        assert_eq!(endianness_flag, data_submessage.endianness_flag());
        assert_eq!(inline_qos_flag, data_submessage.inline_qos_flag());
        assert_eq!(data_flag, data_submessage.data_flag());
        assert_eq!(key_flag, data_submessage.key_flag());
        assert_eq!(
            non_standard_payload_flag,
            data_submessage.non_standard_payload_flag()
        );
        assert_eq!(reader_id, data_submessage.reader_id());
        assert_eq!(writer_id, data_submessage.writer_id());
        assert_eq!(writer_sn, data_submessage.writer_sn());
        assert_eq!(inline_qos, data_submessage.inline_qos());
        assert_eq!(serialized_payload, data_submessage.serialized_payload());
    }

    #[test]
    fn deserialize_no_inline_qos_with_serialized_payload() {
        let inline_qos = ParameterList::empty();
        let serialized_payload = SerializedPayload::new(&[1, 2, 3, 4]);

        #[rustfmt::skip]
        let data_submessage = DataSubmessageRead::new(&[
            0x15, 0b_0000_0101, 24, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            1, 2, 3, 4, // SerializedPayload
        ]);
        assert_eq!(inline_qos, data_submessage.inline_qos());
        assert_eq!(serialized_payload, data_submessage.serialized_payload());
    }

    #[test]
    fn deserialize_with_inline_qos_no_serialized_payload() {
        let inline_qos = ParameterList::new(vec![
            Parameter::new(ParameterId(6), vec![10, 11, 12, 13]),
            Parameter::new(ParameterId(7), vec![20, 21, 22, 23]),
        ]);
        let serialized_payload = SerializedPayload::new(&[]);

        #[rustfmt::skip]
        let data_submessage = DataSubmessageRead::new(&[
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
            1, 0, 1, 0, // inlineQos: Sentinel
        ]);
        assert_eq!(inline_qos, data_submessage.inline_qos());
        assert_eq!(serialized_payload, data_submessage.serialized_payload());
    }

    #[test]
    fn deserialize_with_inline_qos_with_serialized_payload() {
        let inline_qos = ParameterList::new(vec![
            Parameter::new(ParameterId(6), vec![10, 11, 12, 13]),
            Parameter::new(ParameterId(7), vec![20, 21, 22, 23]),
        ]);
        let serialized_payload = SerializedPayload::new(&[1, 2, 3, 4]);

        #[rustfmt::skip]
        let data_submessage = DataSubmessageRead::new(&[
            0x15, 0b_0000_0111, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 1, 0, // inlineQos: Sentinel
            1, 2, 3, 4, // SerializedPayload
        ]);
        assert_eq!(inline_qos, data_submessage.inline_qos());
        assert_eq!(serialized_payload, data_submessage.serialized_payload());
    }
}
