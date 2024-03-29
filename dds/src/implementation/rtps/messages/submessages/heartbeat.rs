use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite},
            submessage_elements::SubmessageElement,
            types::{Count, SubmessageFlag, SubmessageKind},
        },
        types::{EntityId, SequenceNumber, TryReadFromBytes, WriteIntoBytes},
    },
    infrastructure::error::DdsResult,
};

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatSubmessageRead {
    final_flag: SubmessageFlag,
    liveliness_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    count: Count,
}

impl HeartbeatSubmessageRead {
    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        mut data: &[u8],
    ) -> DdsResult<Self> {
        let endianness = submessage_header.endianness();
        Ok(Self {
            final_flag: submessage_header.flags()[1],
            liveliness_flag: submessage_header.flags()[2],
            reader_id: EntityId::try_read_from_bytes(&mut data, endianness)?,
            writer_id: EntityId::try_read_from_bytes(&mut data, endianness)?,
            first_sn: SequenceNumber::try_read_from_bytes(&mut data, endianness)?,
            last_sn: SequenceNumber::try_read_from_bytes(&mut data, endianness)?,
            count: Count::try_read_from_bytes(&mut data, endianness)?,
        })
    }

    pub fn final_flag(&self) -> bool {
        self.final_flag
    }

    pub fn liveliness_flag(&self) -> bool {
        self.liveliness_flag
    }

    pub fn _reader_id(&self) -> EntityId {
        self.reader_id
    }

    pub fn writer_id(&self) -> EntityId {
        self.writer_id
    }

    pub fn first_sn(&self) -> SequenceNumber {
        self.first_sn
    }

    pub fn last_sn(&self) -> SequenceNumber {
        self.last_sn
    }

    pub fn count(&self) -> Count {
        self.count
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatSubmessageWrite<'a> {
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

impl SubmessageHeader for &HeartbeatSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::HEARTBEAT,
            &[self.final_flag, self.liveliness_flag],
            octets_to_next_header,
        )
    }
}

impl WriteIntoBytes for &HeartbeatSubmessageWrite<'_> {
    fn write_into_bytes(&self, buf: &mut &mut [u8]) {
        for submessage_element in &self.submessage_elements {
            submessage_element.write_into_bytes(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::{write_into_bytes_vec, RtpsSubmessageWriteKind},
        types::{USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_heart_beat() {
        let final_flag = false;
        let liveliness_flag = true;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let first_sn = 5;
        let last_sn = 7;
        let count = 2;
        let submessage = RtpsSubmessageWriteKind::Heartbeat(HeartbeatSubmessageWrite::new(
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        ));
        #[rustfmt::skip]
        assert_eq!(write_into_bytes_vec(submessage), vec![
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
        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let expected_first_sn = 5;
        let expected_last_sn = 7;
        let expected_count = 2;
        #[rustfmt::skip]
        let mut data = &[
            0x07, 0b_0000_0101, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage = HeartbeatSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();
        assert_eq!(expected_final_flag, submessage.final_flag());
        assert_eq!(expected_liveliness_flag, submessage.liveliness_flag());
        assert_eq!(expected_reader_id, submessage._reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_first_sn, submessage.first_sn());
        assert_eq!(expected_last_sn, submessage.last_sn());
        assert_eq!(expected_count, submessage.count());
    }
}
