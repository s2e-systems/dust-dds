use crate::implementation::rtps::messages::{
    overall_structure::{RtpsMap, SubmessageHeader, SubmessageHeaderRead},
    types::{SubmessageFlag, Time, TIME_INVALID},
};

#[derive(Debug, PartialEq, Eq)]
pub struct InfoTimestampSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for InfoTimestampSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> InfoTimestampSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn invalidate_flag(&self) -> bool {
        self.submessage_header().flags()[1]
    }

    pub fn timestamp(&self) -> Time {
        if self.invalidate_flag() {
            TIME_INVALID
        } else {
            self.map(&self.data[4..])
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoTimestampSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub invalidate_flag: SubmessageFlag,
    pub timestamp: Time,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deserialize_info_timestamp_valid_time() {
        #[rustfmt::skip]
        let submessage = InfoTimestampSubmessageRead::new(&[
            0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
            4, 0, 0, 0, // Time
            0, 0, 0, 0, // Time
        ]);

        let expected_invalidate_flag = false;
        let expected_timestamp = Time::new(4, 0);

        assert_eq!(expected_invalidate_flag, submessage.invalidate_flag());
        assert_eq!(expected_timestamp, submessage.timestamp());
    }

    #[test]
    fn deserialize_info_timestamp_invalid_time() {
        #[rustfmt::skip]
        let submessage = InfoTimestampSubmessageRead::new(&[
            0x09_u8, 0b_0000_0011, 0, 0, // Submessage header
        ]);

        let expected_invalidate_flag = true;
        let expected_timestamp = TIME_INVALID;

        assert_eq!(expected_invalidate_flag, submessage.invalidate_flag());
        assert_eq!(expected_timestamp, submessage.timestamp());
    }
}
