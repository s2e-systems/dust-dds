use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{Submessage, SubmessageHeaderRead, SubmessageHeaderWrite},
            submessage_elements::SubmessageElement,
            types::{Count, FragmentNumber, SubmessageKind},
        },
        types::{EntityId, SequenceNumber, TryReadFromBytes},
    },
    infrastructure::error::DdsResult,
};

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessageRead {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    last_fragment_num: FragmentNumber,
    count: Count,
}

impl HeartbeatFragSubmessageRead {
    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        mut data: &[u8],
    ) -> DdsResult<Self> {
        let endianness = submessage_header.endianness();
        Ok(Self {
            reader_id: EntityId::try_read_from_bytes(&mut data, endianness)?,
            writer_id: EntityId::try_read_from_bytes(&mut data, endianness)?,
            writer_sn: SequenceNumber::try_read_from_bytes(&mut data, endianness)?,
            last_fragment_num: FragmentNumber::try_read_from_bytes(&mut data, endianness)?,
            count: Count::try_read_from_bytes(&mut data, endianness)?,
        })
    }

    pub fn _reader_id(&self) -> EntityId {
        self.reader_id
    }

    pub fn writer_id(&self) -> EntityId {
        self.writer_id
    }

    pub fn _writer_sn(&self) -> SequenceNumber {
        self.writer_sn
    }

    pub fn _last_fragment_num(&self) -> FragmentNumber {
        self.last_fragment_num
    }

    pub fn count(&self) -> Count {
        self.count
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessageWrite<'a> {
    submessage_elements: [SubmessageElement<'a>; 5],
}
impl HeartbeatFragSubmessageWrite<'_> {
    pub fn _new(
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

impl<'a> Submessage<'a> for HeartbeatFragSubmessageWrite<'a> {
    type SubmessageList = &'a [SubmessageElement<'a>];

    fn submessage_header(
        &self,
        octets_to_next_header: u16,
    ) -> crate::implementation::rtps::messages::overall_structure::SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::HEARTBEAT_FRAG, &[], octets_to_next_header)
    }

    fn submessage_elements(&'a self) -> Self::SubmessageList {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::{
            into_bytes_vec, RtpsSubmessageWriteKind, SubmessageHeaderRead,
        },
        types::{USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_heart_beat() {
        let submessage =
            RtpsSubmessageWriteKind::HeartbeatFrag(HeartbeatFragSubmessageWrite::_new(
                EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
                EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
                5,
                7,
                2,
            ));
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
        let mut data = &[
            0x13_u8, 0b_0000_0001, 24, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: SequenceNumber: high
            5, 0, 0, 0, // writerSN: SequenceNumber: low
            7, 0, 0, 0, // lastFragmentNum
            2, 0, 0, 0, // count: Count
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            HeartbeatFragSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let expected_writer_sn = 5;
        let expected_last_fragment_num = 7;
        let expected_count = 2;

        assert_eq!(expected_reader_id, submessage._reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_writer_sn, submessage._writer_sn());
        assert_eq!(expected_last_fragment_num, submessage._last_fragment_num());
        assert_eq!(expected_count, submessage.count());
    }
}
