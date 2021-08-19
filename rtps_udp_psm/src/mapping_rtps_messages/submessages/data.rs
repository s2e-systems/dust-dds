use std::{io::Write, iter::FromIterator, marker::PhantomData};

use byteorder::ByteOrder;
use rust_rtps_pim::messages::{
    submessage_elements::{
        Parameter, ParameterListSubmessageElement, SerializedDataSubmessageElement,
    },
    submessages::DataSubmessage,
    types::SubmessageKind,
    RtpsSubmessageHeader,
};

use crate::{
    deserialize::Deserialize,
    serialize::{NumberofBytes, Serialize},
};

impl<'a, T> Serialize for DataSubmessage<'a, T>
where
    for<'b> &'b T: IntoIterator<Item = &'b Parameter<'a>>,
{
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        let inline_qos_len = if self.inline_qos_flag {
            self.inline_qos.number_of_bytes()
        } else {
            0
        };
        let serialized_payload_len_padded = self.serialized_payload.number_of_bytes() + 3 & !3; //ceil to multiple of 4
        let submessage_length = 20 + inline_qos_len + serialized_payload_len_padded;

        let header = RtpsSubmessageHeader {
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
            submessage_length: submessage_length as u16,
        };
        header.serialize::<_, B>(&mut writer)?;
        const OCTETS_TO_INLIE_QOS: u16 = 16;
        const EXTRA_FLAGS: u16 = 0;
        EXTRA_FLAGS.serialize::<_, B>(&mut writer)?;
        OCTETS_TO_INLIE_QOS.serialize::<_, B>(&mut writer)?;
        self.reader_id.serialize::<_, B>(&mut writer)?;
        self.writer_id.serialize::<_, B>(&mut writer)?;
        self.writer_sn.serialize::<_, B>(&mut writer)?;
        if self.inline_qos_flag {
            self.inline_qos.serialize::<_, B>(&mut writer)?;
        }
        if self.data_flag || self.key_flag {
            self.serialized_payload.serialize::<_, B>(&mut writer)?;
            // Pad to 32bit boundary
            let padding: &[u8] = match self.serialized_payload.number_of_bytes() % 4 {
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

impl<'de: 'a, 'a, T> Deserialize<'de> for DataSubmessage<'a, T>
where
    T: FromIterator<Parameter<'a>>,
    for<'b> &'b T: IntoIterator<Item = &'b Parameter<'a>>,
{
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let header: RtpsSubmessageHeader = Deserialize::deserialize::<B>(buf)?;
        let inline_qos_flag = header.flags[1];
        let data_flag = header.flags[2];
        let key_flag = header.flags[3];
        let _extra_flags: u16 = Deserialize::deserialize::<B>(buf)?;
        let octets_to_inline_qos: u16 = Deserialize::deserialize::<B>(buf)?;
        let reader_id = Deserialize::deserialize::<B>(buf)?;
        let writer_id = Deserialize::deserialize::<B>(buf)?;
        let writer_sn = Deserialize::deserialize::<B>(buf)?;

        let inline_qos = if inline_qos_flag {
            Deserialize::deserialize::<B>(buf)?
        } else {
            ParameterListSubmessageElement {
                parameter: T::from_iter(std::iter::empty()),
                phantom: PhantomData,
            }
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
            SerializedDataSubmessageElement { value: &data }
        } else {
            SerializedDataSubmessageElement { value: &[] }
        };

        Ok(Self {
            endianness_flag: header.flags[0],
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag: header.flags[4],
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
    use std::marker::PhantomData;

    use crate::{deserialize::from_bytes_le, serialize::to_bytes_le};

    use super::*;
    use rust_rtps_pim::{messages::{submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        }, types::ParameterId}, structure::types::{EntityId, EntityKind}};

    #[test]
    fn serialize_no_inline_qos_no_serialized_payload() {
        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![],
            phantom: PhantomData,
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[] };
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
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter::new(ParameterId(6), &[10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), &[20, 21, 22, 23]);
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
            phantom: PhantomData
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[] };

        let submessage = DataSubmessage{
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
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![],
            phantom: PhantomData
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[1_u8, 2, 3, 4] };
        let submessage = DataSubmessage{
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
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![],
            phantom: PhantomData
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[1_u8, 2, 3] };
        let submessage = DataSubmessage{
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
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![],
            phantom: PhantomData,
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[] };
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
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![],
            phantom: PhantomData,
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[1, 2, 3, 4] };
        let expected = DataSubmessage{
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
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter::new(ParameterId(6), &[10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), &[20, 21, 22, 23]);
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
            phantom: PhantomData
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[] };
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
