use std::io::{Error, Write};

use byteorder::{ByteOrder, ReadBytesExt};

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader,
        submessages::{DataFragSubmessageRead, DataFragSubmessageWrite},
        types::{SerializedPayload, SubmessageKind},
    },
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for DataFragSubmessageWrite<'_> {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        let inline_qos_len = if self.inline_qos_flag {
            self.inline_qos.number_of_bytes()
        } else {
            0
        };
        let serialized_payload_len_padded = (self.serialized_payload.number_of_bytes() + 3) & !3; //ceil to multiple of 4
        let octets_to_next_header = 32 + inline_qos_len + serialized_payload_len_padded;
        RtpsSubmessageHeader {
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

impl<'de: 'a, 'a> MappingReadSubmessage<'de> for DataFragSubmessageRead<'a> {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        let endianness_flag = header.flags[0];
        let inline_qos_flag = header.flags[1];
        let key_flag = header.flags[3];
        let non_standard_payload_flag = header.flags[4];
        let _extra_flags: u16 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let octets_to_inline_qos: u16 =
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let reader_id = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let writer_id = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let writer_sn = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let fragment_starting_num = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let fragments_in_submessage = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let fragment_size = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let data_size = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;

        let mut parameter_list_buf = *buf;
        let parameter_list_buf_length = parameter_list_buf.len();
        let inline_qos_len = if inline_qos_flag {
            loop {
                let pid = parameter_list_buf.read_u16::<B>().expect("pid read failed");
                let length = parameter_list_buf
                    .read_i16::<B>()
                    .expect("length read failed");
                if pid == PID_SENTINEL {
                    break;
                } else {
                    (_, parameter_list_buf) = parameter_list_buf.split_at(length as usize);
                }
            }
            parameter_list_buf_length - parameter_list_buf.len()
        } else {
            0
        };

        let (inline_qos, following) = buf.split_at(inline_qos_len);
        *buf = following;

        let serialized_payload_length =
            header.submessage_length as usize - octets_to_inline_qos as usize - 4 - inline_qos_len;
        let (data, following) = buf.split_at(serialized_payload_length);
        *buf = following;
        let serialized_payload = SerializedPayload::new(data);

        Ok(Self {
            endianness_flag,
            inline_qos_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_starting_num,
            fragments_in_submessage,
            data_size,
            fragment_size,
            inline_qos,
            serialized_payload,
        })
        // todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            history_cache::{RtpsParameter, RtpsParameterList},
            messages::types::{FragmentNumber, ParameterId, SerializedPayload, ULong, UShort},
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
            inline_qos: &RtpsParameterList::empty(),
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
            inline_qos: &RtpsParameterList::new(vec![RtpsParameter::new(
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

    #[test]
    fn deserialize_no_inline_qos_no_serialized_payload() {
        let expected = DataFragSubmessageRead {
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
            inline_qos: &[],
            serialized_payload: SerializedPayload::new(&[]),
        };
        #[rustfmt::skip]
        let result = from_bytes(&[
            0x16_u8, 0b_0000_0001, 32, 0, // Submessage header
            0, 0, 28, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            2, 0, 0, 0, // fragmentStartingNum
            3, 0, 5, 0, // fragmentsInSubmessage | fragmentSize
            4, 0, 0, 0, // sampleSize
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_with_inline_qos_with_serialized_payload() {
        let expected = DataFragSubmessageRead {
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
            inline_qos: &[
                8, 0, 4, 0, // inlineQos: parameterId, length
                71, 72, 73, 74, // inlineQos: value[length]
                1, 0, 0, 0, // inlineQos: Sentinel
            ],
            serialized_payload: SerializedPayload::new(&[1, 2, 3, 0]),
        };
        #[rustfmt::skip]
        let result = from_bytes(&[
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
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
