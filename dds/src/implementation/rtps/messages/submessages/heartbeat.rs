use crate::implementation::rtps::{
    messages::{
        overall_structure::SubmessageHeaderRead, types::SubmessageFlag, RtpsMap, SubmessageHeader,
    },
    types::{Count, EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for HeartbeatSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> HeartbeatSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn final_flag(&self) -> bool {
        self.submessage_header().flags()[1]
    }

    pub fn liveliness_flag(&self) -> bool {
        self.submessage_header().flags()[2]
    }

    pub fn reader_id(&self) -> EntityId {
        self.map(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.map(&self.data[8..])
    }

    pub fn first_sn(&self) -> SequenceNumber {
        self.map(&self.data[12..])
    }

    pub fn last_sn(&self) -> SequenceNumber {
        self.map(&self.data[20..])
    }

    pub fn count(&self) -> Count {
        self.map(&self.data[28..])
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub first_sn: SequenceNumber,
    pub last_sn: SequenceNumber,
    pub count: Count,
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::types::{
        EntityKey, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY,
    };

    use super::*;
    #[test]
    fn deserialize_heart_beat() {
        let expected_final_flag = false;
        let expected_liveliness_flag = true;
        let expected_reader_id =
            EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let expected_writer_id =
            EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let expected_first_sn = SequenceNumber::new(5);
        let expected_last_sn = SequenceNumber::new(7);
        let expected_count = Count::new(2);
        #[rustfmt::skip]
        let submessage = HeartbeatSubmessageRead::new(&[
            0x07, 0b_0000_0101, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ]);
        assert_eq!(expected_final_flag, submessage.final_flag());
        assert_eq!(expected_liveliness_flag, submessage.liveliness_flag());
        assert_eq!(expected_reader_id, submessage.reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_first_sn, submessage.first_sn());
        assert_eq!(expected_last_sn, submessage.last_sn());
        assert_eq!(expected_count, submessage.count());
    }
}
