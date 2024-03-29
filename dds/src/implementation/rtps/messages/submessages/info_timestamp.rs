use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite},
            submessage_elements::SubmessageElement,
            types::{SubmessageFlag, SubmessageKind, Time, TIME_INVALID},
        },
        types::WriteIntoBytes,
    },
    infrastructure::error::DdsResult,
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
    ) -> DdsResult<Self> {
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
#[derive(Debug, PartialEq, Eq)]
pub struct InfoTimestampSubmessageWrite<'a> {
    pub invalidate_flag: SubmessageFlag,
    timestamp_submessage_element: Option<SubmessageElement<'a>>,
}

impl InfoTimestampSubmessageWrite<'_> {
    pub fn new(invalidate_flag: SubmessageFlag, timestamp: Time) -> Self {
        let timestamp_submessage_element = if !invalidate_flag {
            Some(SubmessageElement::Timestamp(timestamp))
        } else {
            None
        };

        Self {
            invalidate_flag,
            timestamp_submessage_element,
        }
    }
}

impl SubmessageHeader for InfoTimestampSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::INFO_TS,
            &[self.invalidate_flag],
            octets_to_next_header,
        )
    }
}

impl WriteIntoBytes for InfoTimestampSubmessageWrite<'_> {
    fn write_into_bytes(&self, buf: &mut &mut [u8]) {
        if let Some(submessage_element) = &self.timestamp_submessage_element {
            submessage_element.write_into_bytes(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::messages::overall_structure::{
        write_into_bytes_vec, RtpsSubmessageWriteKind, SubmessageHeaderRead,
    };

    #[test]
    fn serialize_info_timestamp_valid_time() {
        let submessage = RtpsSubmessageWriteKind::InfoTimestamp(InfoTimestampSubmessageWrite::new(
            false,
            Time::new(4, 0),
        ));
        #[rustfmt::skip]
        assert_eq!(write_into_bytes_vec(submessage), vec![
                0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
                4, 0, 0, 0, // Time
                0, 0, 0, 0, // Time
            ]
        );
    }

    #[test]
    fn serialize_info_timestamp_invalid_time() {
        let submessage = RtpsSubmessageWriteKind::InfoTimestamp(InfoTimestampSubmessageWrite::new(
            true,
            TIME_INVALID,
        ));
        #[rustfmt::skip]
        assert_eq!(write_into_bytes_vec(submessage), vec![
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
