use rust_rtps_pim::messages::types::{SubmessageFlag, SubmessageKindPIM};

use crate::{
    psm::RtpsUdpPsm,
    submessage_elements::{EntityId, SequenceNumber, SequenceNumberSet},
    submessage_header::SubmessageHeader
};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GapSubmessage {
    pub header: SubmessageHeader,
    reader_id: EntityId,
    writer_id: EntityId,
    gap_start: SequenceNumber,
    gap_list: SequenceNumberSet,
}

impl rust_rtps_pim::messages::submessages::GapSubmessage<RtpsUdpPsm> for GapSubmessage {
    fn new(
        endianness_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
    ) -> Self {
        let flags = [endianness_flag].into();

        let submessage_length = 16 + gap_list.len();

        let header = SubmessageHeader {
            submessage_id: RtpsUdpPsm::GAP.into(),
            flags,
            submessage_length,
        };
        Self {
            header,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.header.flags.is_bit_set(0)
    }

    fn reader_id(&self) -> &EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &EntityId {
        &self.writer_id
    }

    fn gap_start(&self) -> &SequenceNumber {
        &self.gap_start
    }

    fn gap_list(&self) -> &SequenceNumberSet {
        &self.gap_list
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for GapSubmessage {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rtps_pim::messages::submessage_elements::{
        SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElementType,
    };
    use rust_serde_cdr::{
        deserializer::RtpsMessageDeserializer, serializer::RtpsMessageSerializer,
    };

    fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
        let mut serializer = RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        };
        value.serialize(&mut serializer).unwrap();
        serializer.writer
    }

    fn deserialize<'de, T: serde::Deserialize<'de>>(buffer: &'de [u8]) -> T {
        let mut de = RtpsMessageDeserializer { reader: buffer };
        serde::de::Deserialize::deserialize(&mut de).unwrap()
    }

    #[test]
    fn serialize_gap() {
        let endianness_flag = true;
        let reader_id = EntityId([1, 2, 3, 4]);
        let writer_id = EntityId([6, 7, 8, 9]);
        let gap_start = SequenceNumber::new(5);
        let gap_list = SequenceNumberSet::new(10, &[]);
        let submessage: GapSubmessage = rust_rtps_pim::messages::submessages::GapSubmessage::new(
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        );
        #[rustfmt::skip]
        assert_eq!(serialize(submessage), vec![
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
        let reader_id = EntityId([1, 2, 3, 4]);
        let writer_id = EntityId([6, 7, 8, 9]);
        let gap_start = SequenceNumber::new(5);
        let gap_list = SequenceNumberSet::new(10, &[]);
        let expected: GapSubmessage = rust_rtps_pim::messages::submessages::GapSubmessage::new(
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        );
        #[rustfmt::skip]
        let result = deserialize(&[
            /*0x08,*/ 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // gapStart: SequenceNumber: high
            5, 0, 0, 0, // gapStart: SequenceNumber: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
           10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
        ]);
        assert_eq!(expected, result);
    }
}
