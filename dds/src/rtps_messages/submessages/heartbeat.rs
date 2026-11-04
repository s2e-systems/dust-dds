use crate::transport::types::{EntityId, SequenceNumber};

use super::super::{
    error::RtpsMessageResult,
    overall_structure::{
        Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, TryReadFromBytes, Write,
        WriteIntoBytes,
    },
    types::{Count, GroupDigest, SubmessageFlag, SubmessageKind},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HeartbeatSubmessage {
    final_flag: SubmessageFlag,
    liveliness_flag: SubmessageFlag,
    group_info_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    count: Count,
    current_gsn: Option<SequenceNumber>,
    first_gsn: Option<SequenceNumber>,
    last_gsn: Option<SequenceNumber>,
    writer_set: Option<GroupDigest>,
}

impl HeartbeatSubmessage {
    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        mut data: &[u8],
    ) -> RtpsMessageResult<Self> {
        let endianness = submessage_header.endianness();
        let final_flag = submessage_header.flags()[1];
        let liveliness_flag = submessage_header.flags()[2];
        let group_info_flag = submessage_header.flags()[3];
        let reader_id = EntityId::try_read_from_bytes(&mut data, endianness)?;
        let writer_id = EntityId::try_read_from_bytes(&mut data, endianness)?;
        let first_sn = SequenceNumber::try_read_from_bytes(&mut data, endianness)?;
        let last_sn = SequenceNumber::try_read_from_bytes(&mut data, endianness)?;
        let count = Count::try_read_from_bytes(&mut data, endianness)?;

        let (current_gsn, first_gsn, last_gsn, writer_set) = if group_info_flag {
            let current_gsn = SequenceNumber::try_read_from_bytes(&mut data, endianness)?;
            let first_gsn = SequenceNumber::try_read_from_bytes(&mut data, endianness)?;
            let last_gsn = SequenceNumber::try_read_from_bytes(&mut data, endianness)?;
            let writer_set = GroupDigest::try_read_from_bytes(&mut data, endianness)?;
            (
                Some(current_gsn),
                Some(first_gsn),
                Some(last_gsn),
                Some(writer_set),
            )
        } else {
            (None, None, None, None)
        };

        Ok(Self {
            final_flag,
            liveliness_flag,
            group_info_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            current_gsn,
            first_gsn,
            last_gsn,
            writer_set,
        })
    }

    pub fn final_flag(&self) -> bool {
        self.final_flag
    }

    pub fn liveliness_flag(&self) -> bool {
        self.liveliness_flag
    }

    pub fn group_info_flag(&self) -> bool {
        self.group_info_flag
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

    pub fn current_gsn(&self) -> Option<SequenceNumber> {
        self.current_gsn
    }

    pub fn first_gsn(&self) -> Option<SequenceNumber> {
        self.first_gsn
    }

    pub fn last_gsn(&self) -> Option<SequenceNumber> {
        self.last_gsn
    }

    pub fn writer_set(&self) -> Option<GroupDigest> {
        self.writer_set
    }
}

impl HeartbeatSubmessage {
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
            group_info_flag: false,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            current_gsn: None,
            first_gsn: None,
            last_gsn: None,
            writer_set: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_group_info(
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        reader_id: EntityId,
        writer_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        count: Count,
        current_gsn: SequenceNumber,
        first_gsn: SequenceNumber,
        last_gsn: SequenceNumber,
        writer_set: GroupDigest,
    ) -> Self {
        Self {
            final_flag,
            liveliness_flag,
            group_info_flag: true,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            current_gsn: Some(current_gsn),
            first_gsn: Some(first_gsn),
            last_gsn: Some(last_gsn),
            writer_set: Some(writer_set),
        }
    }
}

impl Submessage for HeartbeatSubmessage {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(
            SubmessageKind::HEARTBEAT,
            &[self.final_flag, self.liveliness_flag, self.group_info_flag],
            octets_to_next_header,
        )
        .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        self.reader_id.write_into_bytes(buf);
        self.writer_id.write_into_bytes(buf);
        self.first_sn.write_into_bytes(buf);
        self.last_sn.write_into_bytes(buf);
        self.count.write_into_bytes(buf);
        if self.group_info_flag {
            if let Some(current_gsn) = self.current_gsn {
                current_gsn.write_into_bytes(buf);
            }
            if let Some(first_gsn) = self.first_gsn {
                first_gsn.write_into_bytes(buf);
            }
            if let Some(last_gsn) = self.last_gsn {
                last_gsn.write_into_bytes(buf);
            }
            if let Some(writer_set) = self.writer_set {
                writer_set.write_into_bytes(buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        rtps_messages::overall_structure::write_submessage_into_bytes_vec,
        transport::types::{USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
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
        let submessage = HeartbeatSubmessage::new(
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        );
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
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
        let submessage = HeartbeatSubmessage::try_from_bytes(&submessage_header, data).unwrap();
        assert_eq!(expected_final_flag, submessage.final_flag());
        assert_eq!(expected_liveliness_flag, submessage.liveliness_flag());
        assert_eq!(expected_reader_id, submessage._reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_first_sn, submessage.first_sn());
        assert_eq!(expected_last_sn, submessage.last_sn());
        assert_eq!(expected_count, submessage.count());
        assert_eq!(None, submessage.current_gsn());
        assert_eq!(None, submessage.first_gsn());
        assert_eq!(None, submessage.last_gsn());
        assert_eq!(None, submessage.writer_set());
    }

    #[test]
    fn serialize_heart_beat_with_group_info() {
        let final_flag = false;
        let liveliness_flag = true;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let first_sn = 5;
        let last_sn = 7;
        let count = 2;
        let current_gsn = 10;
        let first_gsn = 3;
        let last_gsn = 12;
        let writer_set = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let submessage = HeartbeatSubmessage::new_with_group_info(
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            current_gsn,
            first_gsn,
            last_gsn,
            writer_set,
        );
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
                0x07_u8, 0b_0000_1101, 68, 0, // Submessage header
                1, 2, 3, 4, // readerId: value[4]
                6, 7, 8, 9, // writerId: value[4]
                0, 0, 0, 0, // firstSN: SequenceNumber: high
                5, 0, 0, 0, // firstSN: SequenceNumber: low
                0, 0, 0, 0, // lastSN: SequenceNumberSet: high
                7, 0, 0, 0, // lastSN: SequenceNumberSet: low
                2, 0, 0, 0, // count: Count: value (long)
                0, 0, 0, 0, // currentGSN: high
                10, 0, 0, 0, // currentGSN: low
                0, 0, 0, 0, // firstGSN: high
                3, 0, 0, 0, // firstGSN: low
                0, 0, 0, 0, // lastGSN: high
                12, 0, 0, 0, // lastGSN: low
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // writerSet
            ]
        );
    }

    #[test]
    fn deserialize_heart_beat_with_group_info() {
        let expected_final_flag = false;
        let expected_liveliness_flag = true;
        let expected_group_info_flag = true;
        let expected_reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let expected_writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let expected_first_sn = 5;
        let expected_last_sn = 7;
        let expected_count = 2;
        let expected_current_gsn = 10;
        let expected_first_gsn = 3;
        let expected_last_gsn = 12;
        let expected_writer_set = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        #[rustfmt::skip]
        let mut data = &[
            0x07, 0b_0000_1101, 68, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
            0, 0, 0, 0, // currentGSN: high
            10, 0, 0, 0, // currentGSN: low
            0, 0, 0, 0, // firstGSN: high
            3, 0, 0, 0, // firstGSN: low
            0, 0, 0, 0, // lastGSN: high
            12, 0, 0, 0, // lastGSN: low
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, // writerSet
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage = HeartbeatSubmessage::try_from_bytes(&submessage_header, data).unwrap();
        assert_eq!(expected_final_flag, submessage.final_flag());
        assert_eq!(expected_liveliness_flag, submessage.liveliness_flag());
        assert_eq!(expected_group_info_flag, submessage.group_info_flag());
        assert_eq!(expected_reader_id, submessage._reader_id());
        assert_eq!(expected_writer_id, submessage.writer_id());
        assert_eq!(expected_first_sn, submessage.first_sn());
        assert_eq!(expected_last_sn, submessage.last_sn());
        assert_eq!(expected_count, submessage.count());
        assert_eq!(Some(expected_current_gsn), submessage.current_gsn());
        assert_eq!(Some(expected_first_gsn), submessage.first_gsn());
        assert_eq!(Some(expected_last_gsn), submessage.last_gsn());
        assert_eq!(Some(expected_writer_set), submessage.writer_set());
    }
}
