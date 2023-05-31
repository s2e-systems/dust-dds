use crate::implementation::rtps::{
    messages::{
        overall_structure::{RtpsMap, SubmessageHeader, SubmessageHeaderRead},
        types::{FragmentNumber, SubmessageFlag},
    },
    types::{Count, EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for HeartbeatFragSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> HeartbeatFragSubmessageRead<'a> {
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

    pub fn last_fragment_num(&self) -> FragmentNumber {
        self.map(&self.data[20..])
    }

    pub fn count(&self) -> Count {
        self.map(&self.data[24..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub last_fragment_num: FragmentNumber,
    pub count: Count,
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::types::{
        EntityKey, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY,
    };

    use super::*;
    #[test]
    fn deserialize_heart_beat_frag() {
        #[rustfmt::skip]
        let submessage = HeartbeatFragSubmessageRead::new(&[
            0x13_u8, 0b_0000_0001, 24, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: SequenceNumber: high
            5, 0, 0, 0, // writerSN: SequenceNumber: low
            7, 0, 0, 0, // lastFragmentNum
            2, 0, 0, 0, // count: Count
        ]);

        let expected_reader_id =
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let expected_writer_id =
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let expected_writer_sn = SequenceNumber::new(5);
        let expected_last_fragment_num = FragmentNumber::new(7);
        let expected_count = Count::new(2);

        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_writer_sn, submessage.writer_sn());
        assert_eq!(expected_last_fragment_num, submessage.last_fragment_num());
        assert_eq!(expected_count, submessage.count());
    }
}
