use crate::implementation::rtps::{
    messages::{
        overall_structure::{
            RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
        },
        submessage_elements::SubmessageElement,
        types::{Count, FragmentNumber, SubmessageKind},
    },
    types::{EntityId, SequenceNumber},
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
pub struct HeartbeatFragSubmessageWrite<'a> {
    submessage_elements: [SubmessageElement<'a>; 5],
}
impl HeartbeatFragSubmessageWrite<'_> {
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        last_fragment_num: FragmentNumber,
        count: Count,
    ) -> Self {
        Self {
            submessage_elements: [
                SubmessageElement::EntityId(reader_id),
                SubmessageElement::EntityId(writer_id),
                SubmessageElement::SequenceNumber(writer_sn),
                SubmessageElement::FragmentNumber(last_fragment_num),
                SubmessageElement::Count(count),
            ],
        }
    }
}

impl Submessage for HeartbeatFragSubmessageWrite<'_> {
    fn submessage_header(
        &self,
        octets_to_next_header: u16,
    ) -> crate::implementation::rtps::messages::overall_structure::SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::HEARTBEAT_FRAG, &[], octets_to_next_header)
    }

    fn submessage_elements(&self) -> &[SubmessageElement] {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::into_bytes_vec,
        types::{USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_heart_beat() {
        let submessage = HeartbeatFragSubmessageWrite::new(
            EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
            EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
            SequenceNumber::new(5),
            FragmentNumber::new(7),
            Count::new(2),
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x13_u8, 0b_0000_0001, 24, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // writerSN: SequenceNumber: high
                5, 0, 0, 0, // writerSN: SequenceNumber: low
                7, 0, 0, 0, // lastFragmentNum
                2, 0, 0, 0, // count: Count
            ]
        );
    }

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

        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
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
