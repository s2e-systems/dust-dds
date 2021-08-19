use std::{io::Write, iter::FromIterator};

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::{RtpsSubmessageHeader, submessage_elements::Parameter, submessages::DataSubmessage, types::SubmessageKind}, structure::types::SequenceNumber};

use crate::{deserialize::Deserialize, serialize::{NumberofBytes, Serialize}};

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

// impl<'de, T> Deserialize<'de> for GapSubmessage<T>
// where
//     T: FromIterator<SequenceNumber>,
// {
//     fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
//     where
//         B: ByteOrder,
//     {
//         let header: RtpsSubmessageHeader = Deserialize::deserialize::<B>(buf)?;
//         let reader_id = Deserialize::deserialize::<B>(buf)?;
//         let writer_id = Deserialize::deserialize::<B>(buf)?;
//         let gap_start = Deserialize::deserialize::<B>(buf)?;
//         let gap_list = Deserialize::deserialize::<B>(buf)?;
//         Ok(Self {
//             endianness_flag: header.flags[0],
//             reader_id,
//             writer_id,
//             gap_start,
//             gap_list,
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use crate::{deserialize::from_bytes_le, serialize::to_bytes_le};

    use super::*;
    use rust_rtps_pim::{messages::submessage_elements::{EntityIdSubmessageElement, ParameterListSubmessageElement, SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement, SerializedDataSubmessageElement}, structure::types::{EntityId, EntityKind, SequenceNumber}};

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
        let writer_sn = SequenceNumberSubmessageElement{ value: 5};
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![],
            phantom: PhantomData
        };
        let serialized_payload = SerializedDataSubmessageElement{ value: &[]};
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

    // #[test]
    // fn serialize_gap() {
    //     let endianness_flag = true;
    //     let reader_id = EntityIdSubmessageElement {
    //         value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
    //     };
    //     let writer_id = EntityIdSubmessageElement {
    //         value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
    //     };
    //     let gap_start = SequenceNumberSubmessageElement { value: 5 };
    //     let gap_list: SequenceNumberSetSubmessageElement<[SequenceNumber; 0]> =
    //         SequenceNumberSetSubmessageElement { base: 10, set: [] };
    //     let submessage = GapSubmessage {
    //         endianness_flag,
    //         reader_id,
    //         writer_id,
    //         gap_start,
    //         gap_list,
    //     };
    //     #[rustfmt::skip]
    //     assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
    //             0x08_u8, 0b_0000_0001, 28, 0, // Submessage header
    //             1, 2, 3, 4, // readerId: value[4]
    //             6, 7, 8, 9, // writerId: value[4]
    //             0, 0, 0, 0, // gapStart: SequenceNumber: high
    //             5, 0, 0, 0, // gapStart: SequenceNumber: low
    //             0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
    //            10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
    //             0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
    //         ]
    //     );
    // }

    // #[test]
    // fn deserialize_gap() {
    //     let endianness_flag = true;
    //     let reader_id = EntityIdSubmessageElement {
    //         value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
    //     };
    //     let writer_id = EntityIdSubmessageElement {
    //         value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
    //     };
    //     let gap_start = SequenceNumberSubmessageElement { value: 5 };
    //     let gap_list = SequenceNumberSetSubmessageElement {
    //         base: 10,
    //         set: vec![],
    //     };
    //     let expected = GapSubmessage {
    //         endianness_flag,
    //         reader_id,
    //         writer_id,
    //         gap_start,
    //         gap_list,
    //     };
    //     #[rustfmt::skip]
    //     let result = from_bytes_le(&[
    //         0x08, 0b_0000_0001, 28, 0, // Submessage header
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // gapStart: SequenceNumber: high
    //         5, 0, 0, 0, // gapStart: SequenceNumber: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
    //        10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
    //     ]).unwrap();
    //     assert_eq!(expected, result);
    // }
}
