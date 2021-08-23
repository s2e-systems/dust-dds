use byteorder::ByteOrder;
use std::io::Write;

use crate::{
    submessage_elements::{
         is_bit_set, EntityIdUdp, SequenceNumberUdp, SerializedDataUdp,
    },
    submessage_header::{SubmessageHeaderUdp},
};

#[derive(Debug, PartialEq)]
pub struct DataSubmesageUdp<'a> {
    pub(crate) header: SubmessageHeaderUdp,
    extra_flags: u16,
    octets_to_inline_qos: u16,
    reader_id: EntityIdUdp,
    writer_id: EntityIdUdp,
    writer_sn: SequenceNumberUdp,
    // inline_qos: ParameterListUdp<'a>,
    serialized_payload: SerializedDataUdp<'a>,
}

impl<'a> crate::serialize::Serialize for DataSubmesageUdp<'a> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
        // self.header.serialize::<_, B>(&mut writer)?;
        // self.extra_flags.serialize::<_, B>(&mut writer)?;
        // self.octets_to_inline_qos.serialize::<_, B>(&mut writer)?;
        // self.reader_id.serialize::<_, B>(&mut writer)?;
        // self.writer_id.serialize::<_, B>(&mut writer)?;
        // self.writer_sn.serialize::<_, B>(&mut writer)?;
        // if self.inline_qos_flag() {
        //     self.inline_qos.serialize::<_, B>(&mut writer)?;
        // }
        // if self.data_flag() || self.key_flag() {
        //     self.serialized_payload.serialize::<_, B>(&mut writer)?;
        //     // Pad to 32bit boundary
        //     let padding: &[u8] = match self.serialized_payload.len() % 4 {
        //         1 => &[0; 3],
        //         2 => &[0; 2],
        //         3 => &[0; 1],
        //         _ => &[],
        //     };
        //     padding.serialize::<_, B>(&mut writer)?;
        // }
        // Ok(())
    }
}
impl<'a, 'de: 'a> crate::deserialize::Deserialize<'de> for DataSubmesageUdp<'a> {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{deserialize::from_bytes_le, parameter_list::ParameterUdp, serialize::to_bytes_le};

//     use super::*;
//     use rust_rtps_pim::messages::submessage_elements::SequenceNumberSubmessageElementType;

//     #[test]
//     fn serialize_no_inline_qos_no_serialized_payload() {
//         let endianness_flag = true;
//         let inline_qos_flag = false;
//         let data_flag = false;
//         let key_flag = false;
//         let non_standard_payload_flag = false;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let writer_sn = SequenceNumberUdp::new(&5);
//         let inline_qos = ParameterListUdp {
//             parameter: vec![].into(),
//         };
//         let data = [];
//         let serialized_payload = SerializedDataUdp(data[..].into());
//         let submessage = DataSubmesageUdp::new(
//             endianness_flag,
//             inline_qos_flag,
//             data_flag,
//             key_flag,
//             non_standard_payload_flag,
//             reader_id,
//             writer_id,
//             writer_sn,
//             inline_qos,
//             serialized_payload,
//         );
//         #[rustfmt::skip]
//         assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
//                 0x15_u8, 0b_0000_0001, 20, 0, // Submessage header
//                 0, 0, 16, 0, // extraFlags, octetsToInlineQos
//                 1, 2, 3, 4, // readerId: value[4]
//                 6, 7, 8, 9, // writerId: value[4]
//                 0, 0, 0, 0, // writerSN: high
//                 5, 0, 0, 0, // writerSN: low
//             ]
//         );
//     }

//     #[test]
//     fn serialize_with_inline_qos_no_serialized_payload() {
//         let endianness_flag = true;
//         let inline_qos_flag = true;
//         let data_flag = false;
//         let key_flag = false;
//         let non_standard_payload_flag = false;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let writer_sn = SequenceNumberUdp::new(&5);
//         let param1 = ParameterUdp::new(6, &[10, 11, 12, 13]);
//         let param2 = ParameterUdp::new(7, &[20, 21, 22, 23]);
//         let inline_qos = ParameterListUdp {
//             parameter: vec![param1.into(), param2.into()],
//         };
//         let data = [];
//         let serialized_payload = SerializedDataUdp(data[..].into());
//         let submessage = DataSubmesageUdp::new(
//             endianness_flag,
//             inline_qos_flag,
//             data_flag,
//             key_flag,
//             non_standard_payload_flag,
//             reader_id,
//             writer_id,
//             writer_sn,
//             inline_qos,
//             serialized_payload,
//         );
//         #[rustfmt::skip]
//         assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
//                 0x15, 0b_0000_0011, 40, 0, // Submessage header
//                 0, 0, 16, 0, // extraFlags, octetsToInlineQos
//                 1, 2, 3, 4, // readerId: value[4]
//                 6, 7, 8, 9, // writerId: value[4]
//                 0, 0, 0, 0, // writerSN: high
//                 5, 0, 0, 0, // writerSN: low
//                 6, 0, 4, 0, // inlineQos: parameterId_1, length_1
//                 10, 11, 12, 13, // inlineQos: value_1[length_1]
//                 7, 0, 4, 0, // inlineQos: parameterId_2, length_2
//                 20, 21, 22, 23, // inlineQos: value_2[length_2]
//                 1, 0, 0, 0, // inlineQos: Sentinel
//             ]
//         );
//     }

//     #[test]
//     fn serialize_no_inline_qos_with_serialized_payload() {
//         let endianness_flag = true;
//         let inline_qos_flag = false;
//         let data_flag = true;
//         let key_flag = false;
//         let non_standard_payload_flag = false;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let writer_sn = SequenceNumberUdp::new(&5);
//         let inline_qos = ParameterListUdp {
//             parameter: vec![].into(),
//         };
//         let data = [1_u8, 2, 3, 4];
//         let serialized_payload = SerializedDataUdp(data[..].into());
//         let submessage = DataSubmesageUdp::new(
//             endianness_flag,
//             inline_qos_flag,
//             data_flag,
//             key_flag,
//             non_standard_payload_flag,
//             reader_id,
//             writer_id,
//             writer_sn,
//             inline_qos,
//             serialized_payload,
//         );
//         #[rustfmt::skip]
//         assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
//                 0x15, 0b_0000_0101, 24, 0, // Submessage header
//                 0, 0, 16, 0, // extraFlags, octetsToInlineQos
//                 1, 2, 3, 4, // readerId: value[4]
//                 6, 7, 8, 9, // writerId: value[4]
//                 0, 0, 0, 0, // writerSN: high
//                 5, 0, 0, 0, // writerSN: low
//                 1, 2, 3, 4, // serialized payload
//             ]
//         );
//     }

//     #[test]
//     fn serialize_no_inline_qos_with_serialized_payload_non_multiple_of_4() {
//         let endianness_flag = true;
//         let inline_qos_flag = false;
//         let data_flag = true;
//         let key_flag = false;
//         let non_standard_payload_flag = false;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let writer_sn = SequenceNumberUdp::new(&5);
//         let inline_qos = ParameterListUdp {
//             parameter: vec![].into(),
//         };
//         let data = [1_u8, 2, 3];
//         let serialized_payload = SerializedDataUdp(data[..].into());
//         let submessage = DataSubmesageUdp::new(
//             endianness_flag,
//             inline_qos_flag,
//             data_flag,
//             key_flag,
//             non_standard_payload_flag,
//             reader_id,
//             writer_id,
//             writer_sn,
//             inline_qos,
//             serialized_payload,
//         );
//         #[rustfmt::skip]
//         assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
//                 0x15, 0b_0000_0101, 24, 0, // Submessage header
//                 0, 0, 16, 0, // extraFlags, octetsToInlineQos
//                 1, 2, 3, 4, // readerId: value[4]
//                 6, 7, 8, 9, // writerId: value[4]
//                 0, 0, 0, 0, // writerSN: high
//                 5, 0, 0, 0, // writerSN: low
//                 1, 2, 3, 0, // serialized payload
//             ]
//         );
//     }

//     #[test]
//     fn deserialize_no_inline_qos_no_serialized_payload() {
//         let endianness_flag = true;
//         let inline_qos_flag = false;
//         let data_flag = false;
//         let key_flag = false;
//         let non_standard_payload_flag = false;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let writer_sn = SequenceNumberUdp::new(&5);
//         let inline_qos = ParameterListUdp { parameter: vec![] };
//         let serialized_payload = SerializedDataUdp(&[]);
//         let expected = DataSubmesageUdp::new(
//             endianness_flag,
//             inline_qos_flag,
//             data_flag,
//             key_flag,
//             non_standard_payload_flag,
//             reader_id,
//             writer_id,
//             writer_sn,
//             inline_qos,
//             serialized_payload,
//         );
//         #[rustfmt::skip]
//         let result = from_bytes_le(&[
//             0x15, 0b_0000_0001, 20, 0, // Submessage header
//             0, 0, 16, 0, // extraFlags, octetsToInlineQos
//             1, 2, 3, 4, // readerId: value[4]
//             6, 7, 8, 9, // writerId: value[4]
//             0, 0, 0, 0, // writerSN: high
//             5, 0, 0, 0, // writerSN: low
//         ]).unwrap();
//         assert_eq!(expected, result);
//     }

//     #[test]
//     fn deserialize_no_inline_qos_with_serialized_payload() {
//         let endianness_flag = true;
//         let inline_qos_flag = false;
//         let data_flag = true;
//         let key_flag = false;
//         let non_standard_payload_flag = false;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let writer_sn = SequenceNumberUdp::new(&5);
//         let inline_qos = ParameterListUdp {
//             parameter: vec![].into(),
//         };
//         let data = [1, 2, 3, 4];
//         let serialized_payload = SerializedDataUdp(data[..].into());
//         let expected = DataSubmesageUdp::new(
//             endianness_flag,
//             inline_qos_flag,
//             data_flag,
//             key_flag,
//             non_standard_payload_flag,
//             reader_id,
//             writer_id,
//             writer_sn,
//             inline_qos,
//             serialized_payload,
//         );
//         #[rustfmt::skip]
//         let result = from_bytes_le(&[
//             0x15, 0b_0000_0101, 24, 0, // Submessage header
//             0, 0, 16, 0, // extraFlags, octetsToInlineQos
//             1, 2, 3, 4, // readerId: value[4]
//             6, 7, 8, 9, // writerId: value[4]
//             0, 0, 0, 0, // writerSN: high
//             5, 0, 0, 0, // writerSN: low
//             1, 2, 3, 4, // SerializedPayload
//         ]).unwrap();
//         assert_eq!(expected, result);
//     }

//     #[test]
//     fn deserialize_with_inline_qos_no_serialized_payload() {
//         let endianness_flag = true;
//         let inline_qos_flag = true;
//         let data_flag = false;
//         let key_flag = false;
//         let non_standard_payload_flag = false;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let writer_sn = SequenceNumberUdp::new(&5);
//         let param1 = ParameterUdp::new(6, &[10, 11, 12, 13]);
//         let param2 = ParameterUdp::new(7, &[20, 21, 22, 23]);
//         let inline_qos = ParameterListUdp {
//             parameter: vec![param1.into(), param2.into()],
//         };
//         let serialized_payload = SerializedDataUdp([][..].into());
//         let expected = DataSubmesageUdp::new(
//             endianness_flag,
//             inline_qos_flag,
//             data_flag,
//             key_flag,
//             non_standard_payload_flag,
//             reader_id,
//             writer_id,
//             writer_sn,
//             inline_qos,
//             serialized_payload,
//         );
//         #[rustfmt::skip]
//         let result = from_bytes_le(&[
//             0x15, 0b_0000_0011, 40, 0, // Submessage header
//             0, 0, 16, 0, // extraFlags, octetsToInlineQos
//             1, 2, 3, 4, // readerId: value[4]
//             6, 7, 8, 9, // writerId: value[4]
//             0, 0, 0, 0, // writerSN: high
//             5, 0, 0, 0, // writerSN: low
//             6, 0, 4, 0, // inlineQos: parameterId_1, length_1
//             10, 11, 12, 13, // inlineQos: value_1[length_1]
//             7, 0, 4, 0, // inlineQos: parameterId_2, length_2
//             20, 21, 22, 23, // inlineQos: value_2[length_2]
//             1, 0, 0, 0, // inlineQos: Sentinel
//         ]).unwrap();
//         assert_eq!(expected, result);
//     }
// }
