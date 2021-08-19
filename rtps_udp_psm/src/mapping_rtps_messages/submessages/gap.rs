use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::{RtpsSubmessageHeader, submessage_elements::SequenceNumberSetSubmessageElement, submessages::GapSubmessage, types::{SubmessageFlag, SubmessageKind}}, structure::types::SequenceNumber};

use crate::{
    submessage_elements::{
        flags_to_byte, is_bit_set, EntityIdUdp, SequenceNumberSetUdp, SequenceNumberUdp,
    },
    submessage_header::{SubmessageHeaderUdp, GAP},
    serialize::NumberofBytes,
};


impl<T> crate::serialize::Serialize for GapSubmessage<T> where
for<'a> &'a T: IntoIterator<Item = &'a SequenceNumber>,{
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        let submessage_length = 16 + self.gap_list.number_of_bytes();

        let header = RtpsSubmessageHeader{
            submessage_id: SubmessageKind::GAP,
            flags: [self.endianness_flag, false, false, false, false, false, false, false],
            submessage_length: submessage_length as u16
        };
        header.serialize::<_, B>(&mut writer)?;
        self.reader_id.serialize::<_, B>(&mut writer)?;
        self.writer_id.serialize::<_, B>(&mut writer)?;
        self.gap_start.serialize::<_, B>(&mut writer)?;
        self.gap_list.serialize::<_, B>(&mut writer)
    }
}
impl<'de, S> crate::deserialize::Deserialize<'de> for GapSubmessage<S> {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        // let header = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        // let reader_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        // let writer_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        // let gap_start = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        // let gap_list = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        // Ok(Self {
        //     header,
        //     reader_id,
        //     writer_id,
        //     gap_start,
        //     gap_list,
        // })
        todo!()
    }
}

// impl rust_rtps_pim::messages::submessages::GapSubmessageTrait for GapSubmessageUdp {
//     type EntityIdSubmessageElementType = EntityIdUdp;
//     type SequenceNumberSubmessageElementType = SequenceNumberUdp;
//     type SequenceNumberSetSubmessageElementType = SequenceNumberSetUdp;

//     fn new(
//         endianness_flag: SubmessageFlag,
//         reader_id: EntityIdUdp,
//         writer_id: EntityIdUdp,
//         gap_start: SequenceNumberUdp,
//         gap_list: SequenceNumberSetUdp,
//     ) -> Self {
//         let flags = flags_to_byte([endianness_flag]);

//         let submessage_length = 16 + gap_list.len();

//         let header = SubmessageHeaderUdp {
//             submessage_id: GAP,
//             flags,
//             submessage_length,
//         };
//         Self {
//             header,
//             reader_id,
//             writer_id,
//             gap_start,
//             gap_list,
//         }
//     }

//     fn endianness_flag(&self) -> SubmessageFlag {
//         is_bit_set(self.header.flags, 0)
//     }

//     fn reader_id(&self) -> &EntityIdUdp {
//         &self.reader_id
//     }

//     fn writer_id(&self) -> &EntityIdUdp {
//         &self.writer_id
//     }

//     fn gap_start(&self) -> &SequenceNumberUdp {
//         &self.gap_start
//     }

//     fn gap_list(&self) -> &SequenceNumberSetUdp {
//         &self.gap_list
//     }
// }


#[cfg(test)]
mod tests {
    use crate::{deserialize::from_bytes_le, serialize::to_bytes_le};

    use super::*;
    use rust_rtps_pim::{messages::submessage_elements::{EntityIdSubmessageElement, SequenceNumberSetSubmessageElement, SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElement, SequenceNumberSubmessageElementType}, structure::types::{EntityId, EntityKind, SequenceNumber}};
    #[test]
    fn serialize_gap() {
        let endianness_flag = true;
        let reader_id = EntityIdSubmessageElement { value: EntityId::new(
            [1, 2, 3],
            EntityKind::UserDefinedReaderNoKey,
        )};
        let writer_id = EntityIdSubmessageElement { value: EntityId::new(
            [6, 7, 8], EntityKind::UserDefinedReaderGroup,
        )};
        let gap_start = SequenceNumberSubmessageElement{value: 5};
        let gap_list: SequenceNumberSetSubmessageElement<[SequenceNumber; 0]> = SequenceNumberSetSubmessageElement{base: 10, set: []};
        let submessage = GapSubmessage {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&submessage).unwrap(), vec![
                0x08_u8, 0b_0000_0001, 28, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // gapStart: SequenceNumber: high
                5, 0, 0, 0, // gapStart: SequenceNumber: low
                0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
               10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
                0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
            ]
        );
    }

    #[test]
    fn deserialize_gap() {
        let endianness_flag = true;
        let reader_id = EntityIdSubmessageElement { value: EntityId::new(
            [1, 2, 3],
            EntityKind::UserDefinedReaderNoKey,
        )};
        let writer_id = EntityIdSubmessageElement { value: EntityId::new(
            [6, 7, 8], EntityKind::UserDefinedReaderGroup,
        )};
        let gap_start = SequenceNumberSubmessageElement{value: 5};
        let gap_list: SequenceNumberSetSubmessageElement<[SequenceNumber; 0]> = SequenceNumberSetSubmessageElement{base: 10, set: []};
        let expected = GapSubmessage {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            0x08, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // gapStart: SequenceNumber: high
            5, 0, 0, 0, // gapStart: SequenceNumber: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
           10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
