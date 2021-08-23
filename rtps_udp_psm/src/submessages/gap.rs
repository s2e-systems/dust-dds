use std::io::Write;

use byteorder::ByteOrder;

use crate::{
    submessage_elements::{
        EntityIdUdp, SequenceNumberSetUdp, SequenceNumberUdp,
    },
    submessage_header::{SubmessageHeaderUdp},
};

#[derive(Debug, PartialEq)]
pub struct GapSubmessageUdp {
    pub header: SubmessageHeaderUdp,
    reader_id: EntityIdUdp,
    writer_id: EntityIdUdp,
    gap_start: SequenceNumberUdp,
    gap_list: SequenceNumberSetUdp,
}

impl crate::serialize::Serialize for GapSubmessageUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.header.serialize::<_, B>(&mut writer)?;
        self.reader_id.serialize::<_, B>(&mut writer)?;
        self.writer_id.serialize::<_, B>(&mut writer)?;
        self.gap_start.serialize::<_, B>(&mut writer)?;
        self.gap_list.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for GapSubmessageUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let header = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let reader_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let writer_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let gap_start = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let gap_list = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self {
            header,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        })
    }
}



// #[cfg(test)]
// mod tests {
//     use crate::{deserialize::from_bytes_le, serialize::to_bytes_le};

//     use super::*;
//     use rust_rtps_pim::messages::submessage_elements::{
//         SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElementType,
//     };
//     #[test]
//     fn serialize_gap() {
//         let endianness_flag = true;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let gap_start = SequenceNumberUdp::new(&5);
//         let gap_list = SequenceNumberSetUdp::new(&10, &[]);
//         let submessage: GapSubmessageUdp = rust_rtps_pim::messages::submessages::GapSubmessageTrait::new(
//             endianness_flag,
//             reader_id,
//             writer_id,
//             gap_start,
//             gap_list,
//         );
//         #[rustfmt::skip]
//         assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
//                 0x08_u8, 0b_0000_0001, 28, 0, // Submessage header
//                 1, 2, 3, 4, // readerId: value[4]
//                 6, 7, 8, 9, // writerId: value[4]
//                 0, 0, 0, 0, // gapStart: SequenceNumber: high
//                 5, 0, 0, 0, // gapStart: SequenceNumber: low
//                 0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
//                10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
//                 0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
//             ]
//         );
//     }

//     #[test]
//     fn deserialize_gap() {
//         let endianness_flag = true;
//         let reader_id = EntityIdUdp {
//             entity_key: [1, 2, 3],
//             entity_kind: 4,
//         };
//         let writer_id = EntityIdUdp {
//             entity_key: [6, 7, 8],
//             entity_kind: 9,
//         };
//         let gap_start = SequenceNumberUdp::new(&5);
//         let gap_list = SequenceNumberSetUdp::new(&10, &[]);
//         let expected: GapSubmessageUdp = rust_rtps_pim::messages::submessages::GapSubmessageTrait::new(
//             endianness_flag,
//             reader_id,
//             writer_id,
//             gap_start,
//             gap_list,
//         );
//         #[rustfmt::skip]
//         let result = from_bytes_le(&[
//             0x08, 0b_0000_0001, 28, 0, // Submessage header
//             1, 2, 3, 4, // readerId: value[4]
//             6, 7, 8, 9, // writerId: value[4]
//             0, 0, 0, 0, // gapStart: SequenceNumber: high
//             5, 0, 0, 0, // gapStart: SequenceNumber: low
//             0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
//            10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
//             0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
//         ]).unwrap();
//         assert_eq!(expected, result);
//     }
// }
