use super::super::{
    error::RtpsMessageResult,
    overall_structure::{
        Submessage, SubmessageHeaderRead, SubmessageHeaderWrite, Write, WriteIntoBytes,
    },
    types::{SubmessageFlag, SubmessageKind, TIME_INVALID, Time},
};

#[derive(Debug, PartialEq, Eq)]
pub struct InfoTimestampSubmessageRead {
    invalidate_flag: SubmessageFlag,
    timestamp: Time,
}

impl InfoTimestampSubmessageRead {
    pub fn try_from_bytes(
        submessage_header: &SubmessageHeaderRead,
        mut data: &[u8],
    ) -> RtpsMessageResult<Self> {
        let invalidate_flag = submessage_header.flags()[1];
        let timestamp = if invalidate_flag {
            TIME_INVALID
        } else {
            Time::try_read_from_bytes(&mut data, submessage_header.endianness())?
        };
        Ok(Self {
            invalidate_flag,
            timestamp,
        })
    }

    pub fn invalidate_flag(&self) -> bool {
        self.invalidate_flag
    }

    pub fn timestamp(&self) -> Time {
        self.timestamp
    }
}

impl InfoTimestampSubmessageRead {
    pub fn new(invalidate_flag: SubmessageFlag, timestamp: Time) -> Self {
        Self {
            invalidate_flag,
            timestamp,
        }
    }
}

impl Submessage for InfoTimestampSubmessageRead {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(
            SubmessageKind::INFO_TS,
            &[self.invalidate_flag],
            octets_to_next_header,
        )
        .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        if !self.invalidate_flag {
            self.timestamp.write_into_bytes(buf);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InfoTimestampSubmessageWrite {
    invalidate_flag: SubmessageFlag,
    timestamp: Time,
}

impl InfoTimestampSubmessageWrite {
    pub fn new(invalidate_flag: SubmessageFlag, timestamp: Time) -> Self {
        Self {
            invalidate_flag,
            timestamp,
        }
    }

    pub fn invalidate_flag(&self) -> bool {
        self.invalidate_flag
    }

    pub fn timestamp(&self) -> Time {
        self.timestamp
    }
}

impl Submessage for InfoTimestampSubmessageWrite {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write) {
        SubmessageHeaderWrite::new(
            SubmessageKind::INFO_TS,
            &[self.invalidate_flag],
            octets_to_next_header,
        )
        .write_into_bytes(buf);
    }

    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write) {
        if !self.invalidate_flag {
            self.timestamp.write_into_bytes(buf);
        }
    }

    fn submessage_len(&self) -> usize {
        if self.invalidate_flag { 4 } else { 12 }
    }

    fn write_submessage_into_bytes(&self, buf: &mut [u8]) -> usize {
        if self.invalidate_flag {
            SubmessageHeaderWrite::new(SubmessageKind::INFO_TS, &[true], 0)
                .write_into_slice(&mut buf[0..4]);
            4
        } else {
            SubmessageHeaderWrite::new(SubmessageKind::INFO_TS, &[false], 8)
                .write_into_slice(&mut buf[0..4]);
            self.timestamp.write_into_slice(&mut buf[4..12]);
            12
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps_messages::overall_structure::write_submessage_into_bytes_vec;

    #[test]
    fn serialize_info_timestamp_valid_time() {
        let submessage = InfoTimestampSubmessageRead::new(false, Time::new(4, 0));
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
                0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
                4, 0, 0, 0, // Time
                0, 0, 0, 0, // Time
            ]
        );
    }

    #[test]
    fn serialize_info_timestamp_invalid_time() {
        let submessage = InfoTimestampSubmessageRead::new(true, TIME_INVALID);
        #[rustfmt::skip]
        assert_eq!(write_submessage_into_bytes_vec(&submessage), vec![
                0x09_u8, 0b_0000_0011, 0, 0, // Submessage header
            ]
        );
    }

    #[test]
    fn deserialize_info_timestamp_valid_time() {
        #[rustfmt::skip]
        let mut data = &[
            0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
            4, 0, 0, 0, // Time
            0, 0, 0, 0, // Time
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            InfoTimestampSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        let expected_invalidate_flag = false;
        let expected_timestamp = Time::new(4, 0);

        assert_eq!(expected_invalidate_flag, submessage.invalidate_flag());
        assert_eq!(expected_timestamp, submessage.timestamp());
    }

    #[test]
    fn deserialize_info_timestamp_invalid_time() {
        #[rustfmt::skip]
        let mut data = &[
            0x09_u8, 0b_0000_0011, 0, 0, // Submessage header
        ][..];
        let submessage_header = SubmessageHeaderRead::try_read_from_bytes(&mut data).unwrap();
        let submessage =
            InfoTimestampSubmessageRead::try_from_bytes(&submessage_header, data).unwrap();

        let expected_invalidate_flag = true;
        let expected_timestamp = TIME_INVALID;

        assert_eq!(expected_invalidate_flag, submessage.invalidate_flag());
        assert_eq!(expected_timestamp, submessage.timestamp());
    }
}
