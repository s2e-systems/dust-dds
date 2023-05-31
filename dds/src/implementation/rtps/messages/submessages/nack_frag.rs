use crate::implementation::rtps::{
    messages::{
        overall_structure::{RtpsMap, SubmessageHeader, SubmessageHeaderRead},
        submessage_elements::FragmentNumberSet,
        types::SubmessageFlag,
    },
    types::{Count, EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for NackFragSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> NackFragSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn reader_id(&self) -> EntityId {
        self.map(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.map(&self.data[8..])
    }

    pub fn writer_sn(&self) -> SequenceNumber {
        self.map(&self.data[12..])
    }

    pub fn fragment_number_state(&self) -> FragmentNumberSet {
        self.map(&self.data[20..])
    }

    pub fn count(&self) -> Count {
        self.map(&self.data[self.data.len() - 4..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_number_state: FragmentNumberSet,
    pub count: Count,
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::{
        messages::types::FragmentNumber,
        types::{EntityKey, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    use super::*;
    #[test]
    fn deserialize_nack_frag() {
        #[rustfmt::skip]
        let submessage = NackFragSubmessageRead::new(&[
            0x12_u8, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN
            4, 0, 0, 0, // writerSN
           10, 0, 0, 0, // fragmentNumberState.base
            0, 0, 0, 0, // fragmentNumberState.numBits
            6, 0, 0, 0, // count
        ]);

        let expected_reader_id =
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let expected_writer_id =
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let expected_writer_sn = SequenceNumber::new(4);
        let expected_fragment_number_state = FragmentNumberSet {
            base: FragmentNumber::new(10),
            set: vec![],
        };
        let expected_count = Count::new(6);

        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_writer_sn, submessage.writer_sn());
        assert_eq!(
            expected_fragment_number_state,
            submessage.fragment_number_state()
        );
        assert_eq!(expected_count, submessage.count());
    }
}
