use crate::implementation::rtps::{
    messages::{
        overall_structure::{
            RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
        },
        submessage_elements::SubmessageElement,
        types::{Count, SubmessageFlag, SubmessageKind},
    },
    types::{EntityId, SequenceNumber},
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
pub struct HeartbeatSubmessageWrite<'a> {
    endianness_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    liveliness_flag: SubmessageFlag,
    submessage_elements: [SubmessageElement<'a>; 5],
}

impl HeartbeatSubmessageWrite<'_> {
    pub fn new(
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        count: Count,
    ) -> Self {
        Self {
            endianness_flag: true,
            final_flag,
            liveliness_flag,
            submessage_elements: [
                SubmessageElement::EntityId(reader_id),
                SubmessageElement::EntityId(writer_id),
                SubmessageElement::SequenceNumber(first_sn),
                SubmessageElement::SequenceNumber(last_sn),
                SubmessageElement::Count(count),
            ],
        }
    }
}

impl Submessage for HeartbeatSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::HEARTBEAT,
            &[self.endianness_flag, self.final_flag, self.liveliness_flag],
            octets_to_next_header,
        )
    }

    fn submessage_elements(&self) -> &[SubmessageElement] {
        &self.submessage_elements
    }

    fn endianness_flag(&self) -> bool {
        self.endianness_flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::into_bytes_vec,
        types::{EntityKey, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_heart_beat() {
        let final_flag = false;
        let liveliness_flag = true;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let first_sn = SequenceNumber::new(5);
        let last_sn = SequenceNumber::new(7);
        let count = Count::new(2);
        let submessage = HeartbeatSubmessageWrite::new(
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x07_u8, 0b_0000_0101, 28, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // firstSN: SequenceNumber: high
                5, 0, 0, 0, // firstSN: SequenceNumber: low
                0, 0, 0, 0, // lastSN: SequenceNumberSet: high
                7, 0, 0, 0, // lastSN: SequenceNumberSet: low
                2, 0, 0, 0, // count: Count: value (long)
            ]
        );
    }

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
