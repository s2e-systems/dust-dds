use byteorder::ByteOrder;
use rust_rtps_pim::messages::{
    submessages::DataSubmessageTrait, types::SubmessageFlag, RtpsSubmessageHeader, Submessage,
};
use std::io::Write;

use crate::{
    parameter_list::ParameterListUdp,
    submessage_elements::{
        flags_to_byte, is_bit_set, EntityIdUdp, SequenceNumberUdp, SerializedDataUdp,
    },
    submessage_header::{SubmessageHeaderUdp, DATA},
};

#[derive(Debug, PartialEq)]
pub struct DataSubmesageUdp<'a> {
    pub(crate) header: SubmessageHeaderUdp,
    extra_flags: u16,
    octets_to_inline_qos: u16,
    reader_id: EntityIdUdp,
    writer_id: EntityIdUdp,
    writer_sn: SequenceNumberUdp,
    inline_qos: ParameterListUdp<'a>,
    serialized_payload: SerializedDataUdp<'a>,
}

impl<'a> crate::serialize::Serialize for DataSubmesageUdp<'a> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.header.serialize::<_, B>(&mut writer)?;
        self.extra_flags.serialize::<_, B>(&mut writer)?;
        self.octets_to_inline_qos.serialize::<_, B>(&mut writer)?;
        self.reader_id.serialize::<_, B>(&mut writer)?;
        self.writer_id.serialize::<_, B>(&mut writer)?;
        self.writer_sn.serialize::<_, B>(&mut writer)?;
        if self.inline_qos_flag() {
            self.inline_qos.serialize::<_, B>(&mut writer)?;
        }
        if self.data_flag() || self.key_flag() {
            self.serialized_payload.serialize::<_, B>(&mut writer)?;
            // Pad to 32bit boundary
            let padding: &[u8] = match self.serialized_payload.len() % 4 {
                1 => &[0; 3],
                2 => &[0; 2],
                3 => &[0; 1],
                _ => &[],
            };
            padding.serialize::<_, B>(&mut writer)?;
        }
        Ok(())
    }
}
impl<'a, 'de:'a> crate::deserialize::Deserialize<'de> for DataSubmesageUdp<'a> {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let header: SubmessageHeaderUdp = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let inline_qos_flag = is_bit_set(header.flags, 1);
        let data_flag = is_bit_set(header.flags, 2);
        let key_flag = is_bit_set(header.flags, 3);
        // let non_standard_payload_flag = header.flags.is_bit_set(4);
        let extra_flags = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let octets_to_inline_qos = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let reader_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let writer_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let writer_sn = crate::deserialize::Deserialize::deserialize::<B>(buf)?;

        let inline_qos: ParameterListUdp = if inline_qos_flag {
            crate::deserialize::Deserialize::deserialize::<B>(buf)?
        } else {
            ParameterListUdp { parameter: vec![] }
        };
        let inline_qos_len = if inline_qos_flag {
            inline_qos.number_of_bytes()
        } else {
            0
        };

        let serialized_payload: SerializedDataUdp = if data_flag || key_flag {
            let serialized_payload_length =
            header.submessage_length as usize - octets_to_inline_qos as usize - 4 - inline_qos_len;
            let (data, following) = buf.split_at(serialized_payload_length as usize);
            *buf = following;

            SerializedDataUdp(&data)
        } else {
            SerializedDataUdp(&[])
        };

        Ok(DataSubmesageUdp {
            header,
            extra_flags,
            octets_to_inline_qos,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        })
    }
}

impl<'a> rust_rtps_pim::messages::submessages::DataSubmessageTrait<'a> for DataSubmesageUdp<'a> {
    type EntityIdSubmessageElementType = EntityIdUdp;
    type SequenceNumberSubmessageElementType = SequenceNumberUdp;
    type ParameterListSubmessageElementType = ParameterListUdp<'a>;
    type SerializedDataSubmessageElementType = SerializedDataUdp<'a>;

    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: Self::EntityIdSubmessageElementType,
        writer_id: Self::EntityIdSubmessageElementType,
        writer_sn: Self::SequenceNumberSubmessageElementType,
        inline_qos: Self::ParameterListSubmessageElementType,
        serialized_payload: Self::SerializedDataSubmessageElementType,
    ) -> Self {
        let flags = flags_to_byte([
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
        ]);
        let inline_qos_len = if inline_qos_flag {
            inline_qos.number_of_bytes()
        } else {
            0
        };
        let serialized_payload_len_padded = serialized_payload.len() + 3 & !3; //ceil to multiple of 4
        let submessage_length = 20 + inline_qos_len + serialized_payload_len_padded;
        let header = SubmessageHeaderUdp {
            submessage_id: DATA,
            flags,
            submessage_length: submessage_length as u16,
        };

        DataSubmesageUdp {
            header,
            extra_flags: 0b_0000_0000_0000_0000,
            octets_to_inline_qos: 16,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        is_bit_set(self.header.flags, 0)
    }

    fn inline_qos_flag(&self) -> SubmessageFlag {
        is_bit_set(self.header.flags, 1)
    }

    fn data_flag(&self) -> SubmessageFlag {
        is_bit_set(self.header.flags, 2)
    }

    fn key_flag(&self) -> SubmessageFlag {
        is_bit_set(self.header.flags, 3)
    }

    fn non_standard_payload_flag(&self) -> SubmessageFlag {
        is_bit_set(self.header.flags, 4)
    }

    fn reader_id(&self) -> &EntityIdUdp {
        &self.reader_id
    }

    fn writer_id(&self) -> &EntityIdUdp {
        &self.writer_id
    }

    fn writer_sn(&self) -> &SequenceNumberUdp {
        &self.writer_sn
    }

    fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType {
        todo!()
    }

    fn serialized_payload(&self) -> &SerializedDataUdp<'a> {
        &self.serialized_payload
    }
}

impl<'a> Submessage for DataSubmesageUdp<'a> {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use crate::{deserialize::from_bytes_le, parameter_list::ParameterUdp, serialize::to_bytes_le};

    use super::*;
    use rust_rtps_pim::messages::submessage_elements::SequenceNumberSubmessageElementType;

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let inline_qos = ParameterListUdp {
            parameter: vec![].into(),
        };
        let data = [];
        let serialized_payload = SerializedDataUdp(data[..].into());
        let submessage = DataSubmesageUdp::new(
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
        );
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
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
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let param1 = ParameterUdp::new(6, &[10, 11, 12, 13]);
        let param2 = ParameterUdp::new(7, &[20, 21, 22, 23]);
        let inline_qos = ParameterListUdp {
            parameter: vec![param1.into(), param2.into()],
        };
        let data = [];
        let serialized_payload = SerializedDataUdp(data[..].into());
        let submessage = DataSubmesageUdp::new(
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
        );
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
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
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let inline_qos = ParameterListUdp {
            parameter: vec![].into(),
        };
        let data = [1_u8, 2, 3, 4];
        let serialized_payload = SerializedDataUdp(data[..].into());
        let submessage = DataSubmesageUdp::new(
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
        );
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
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
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let inline_qos = ParameterListUdp {
            parameter: vec![].into(),
        };
        let data = [1_u8, 2, 3];
        let serialized_payload = SerializedDataUdp(data[..].into());
        let submessage = DataSubmesageUdp::new(
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
        );
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
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
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let inline_qos = ParameterListUdp { parameter: vec![] };
        let serialized_payload = SerializedDataUdp(&[]);
        let expected = DataSubmesageUdp::new(
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
        );
        #[rustfmt::skip]
        let result = from_bytes_le(&[
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
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let inline_qos = ParameterListUdp {
            parameter: vec![].into(),
        };
        let data = [1, 2, 3, 4];
        let serialized_payload = SerializedDataUdp(data[..].into());
        let expected = DataSubmesageUdp::new(
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
        );
        #[rustfmt::skip]
        let result = from_bytes_le(&[
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
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let param1 = ParameterUdp::new(6, &[10, 11, 12, 13]);
        let param2 = ParameterUdp::new(7, &[20, 21, 22, 23]);
        let inline_qos = ParameterListUdp {
            parameter: vec![param1.into(), param2.into()],
        };
        let serialized_payload = SerializedDataUdp([][..].into());
        let expected = DataSubmesageUdp::new(
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
        );
        #[rustfmt::skip]
        let result = from_bytes_le(&[
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
