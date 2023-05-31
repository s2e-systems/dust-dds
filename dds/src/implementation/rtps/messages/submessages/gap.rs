use crate::implementation::rtps::{
    messages::{
        overall_structure::{RtpsMap, SubmessageHeader, SubmessageHeaderRead},
        submessage_elements::SequenceNumberSet,
        types::SubmessageFlag,
    },
    types::{EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for GapSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> GapSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn reader_id(&self) -> EntityId {
        self.map(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.map(&self.data[8..])
    }

    pub fn gap_start(&self) -> SequenceNumber {
        self.map(&self.data[12..])
    }

    pub fn gap_list(&self) -> SequenceNumberSet {
        self.map(&self.data[20..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub gap_start: SequenceNumber,
    pub gap_list: SequenceNumberSet,
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::types::{
        EntityKey, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY,
    };

    use super::*;
    #[test]
    fn deserialize_gap() {
        let expected_reader_id =
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let expected_writer_id =
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let expected_gap_start = SequenceNumber::new(5);
        let expected_gap_list = SequenceNumberSet {
            base: SequenceNumber::new(10),
            set: vec![],
        };
        #[rustfmt::skip]
        let submessage = GapSubmessageRead::new(&[
            0x08, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // gapStart: SequenceNumber: high
            5, 0, 0, 0, // gapStart: SequenceNumber: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
           10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
        ]);
        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_gap_start, submessage.gap_start());
        assert_eq!(expected_gap_list, submessage.gap_list());
    }
}
