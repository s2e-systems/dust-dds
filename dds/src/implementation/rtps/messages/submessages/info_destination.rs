use crate::{
    implementation::rtps::{
        messages::{
            overall_structure::{Submessage, SubmessageHeaderWrite},
            submessage_elements::SubmessageElement,
            types::SubmessageKind,
        },
        types::{Endianness, GuidPrefix, TryFromBytes},
    },
    infrastructure::error::{DdsError, DdsResult},
};

#[derive(Debug, PartialEq, Eq)]
pub struct InfoDestinationSubmessageRead {
    guid_prefix: GuidPrefix,
}

impl InfoDestinationSubmessageRead {
    pub fn try_from_bytes(data: &[u8]) -> DdsResult<Self> {
        if data.len() >= 16 {
            let endianness = &Endianness::from_flags(data[1]);
            Ok(Self {
                guid_prefix: GuidPrefix::try_from_bytes(&data[4..], endianness)?,
            })
        } else {
            Err(DdsError::Error(
                "InfoDestination submessage invalid".to_string(),
            ))
        }
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct InfoDestinationSubmessageWrite<'a> {
    submessage_elements: [SubmessageElement<'a>; 1],
}

impl InfoDestinationSubmessageWrite<'_> {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        Self {
            submessage_elements: [SubmessageElement::GuidPrefix(guid_prefix)],
        }
    }
}

impl<'a> Submessage<'a> for InfoDestinationSubmessageWrite<'a> {
    type SubmessageList = &'a [SubmessageElement<'a>];

    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::INFO_DST, &[], octets_to_next_header)
    }

    fn submessage_elements(&'a self) -> Self::SubmessageList {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::{into_bytes_vec, RtpsSubmessageWriteKind},
        types::GUIDPREFIX_UNKNOWN,
    };

    #[test]
    fn serialize_heart_beat() {
        let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let submessage = RtpsSubmessageWriteKind::InfoDestination(
            InfoDestinationSubmessageWrite::new(guid_prefix),
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
              0x0e, 0b_0000_0001, 12, 0, // Submessage header
                1, 2, 3, 4, //guid_prefix
                5, 6, 7, 8, //guid_prefix
                9, 10, 11, 12, //guid_prefix
            ]
        );
    }

    #[test]
    fn deserialize_info_destination() {
        #[rustfmt::skip]
        let submessage = InfoDestinationSubmessageRead::try_from_bytes(&[
            0x0e, 0b_0000_0001, 12, 0, // Submessage header
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
        ]).unwrap();

        let expected_guid_prefix = GUIDPREFIX_UNKNOWN;
        assert_eq!(expected_guid_prefix, submessage.guid_prefix());
    }
}
