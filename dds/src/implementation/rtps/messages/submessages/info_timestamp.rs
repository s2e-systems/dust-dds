use crate::implementation::rtps::messages::{
    overall_structure::{
        RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
    },
    submessage_elements::SubmessageElement,
    types::{SubmessageFlag, SubmessageKind, Time, TIME_INVALID},
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
pub struct InfoTimestampSubmessageWrite<'a> {
    pub endianness_flag: SubmessageFlag,
    pub invalidate_flag: SubmessageFlag,
    submessage_elements: Vec<SubmessageElement<'a>>,
}

impl InfoTimestampSubmessageWrite<'_> {
    pub fn new(
        invalidate_flag: SubmessageFlag,
        timestamp: Time,
    ) -> Self {
        let mut submessage_elements = vec![];
        if !invalidate_flag {
            submessage_elements.push(SubmessageElement::Timestamp(timestamp))
        }
        Self {
            endianness_flag: true,
            invalidate_flag,
            submessage_elements,
        }
    }
}

impl Submessage for InfoTimestampSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::INFO_TS,
            &[self.endianness_flag, self.invalidate_flag],
            octets_to_next_header,
        )
    }

    fn submessage_elements(&self) -> &[SubmessageElement] {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::messages::overall_structure::into_bytes_vec;

    #[test]
    fn serialize_info_timestamp_valid_time() {
        let submessage = InfoTimestampSubmessageWrite::new(false, Time::new(4, 0));
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
                4, 0, 0, 0, // Time
                0, 0, 0, 0, // Time
            ]
        );
    }

    #[test]
    fn serialize_info_timestamp_invalid_time() {
        let submessage = InfoTimestampSubmessageWrite::new(true, TIME_INVALID);
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x09_u8, 0b_0000_0011, 0, 0, // Submessage header
            ]
        );
    }

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
