use crate::implementation::rtps::{
    messages::{
        overall_structure::{
            RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
        },
        submessage_elements::SubmessageElement,
        types::{SubmessageFlag, SubmessageKind},
    },
    types::GuidPrefix,
};

#[derive(Debug, PartialEq, Eq)]
pub struct InfoDestinationSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for InfoDestinationSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> InfoDestinationSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.map(&self.data[4..])
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct InfoDestinationSubmessageWrite<'a> {
    endianness_flag: SubmessageFlag,
    submessage_elements: [SubmessageElement<'a>; 1],
}

impl InfoDestinationSubmessageWrite<'_> {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        Self {
            endianness_flag: true,
            submessage_elements: [SubmessageElement::GuidPrefix(guid_prefix)],
        }
    }
}

impl Submessage for InfoDestinationSubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(
            SubmessageKind::INFO_DST,
            &[self.endianness_flag],
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
        messages::overall_structure::into_bytes_vec, types::GUIDPREFIX_UNKNOWN,
    };

    #[test]
    fn serialize_heart_beat() {
        let guid_prefix = GuidPrefix::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let submessage = InfoDestinationSubmessageWrite::new(guid_prefix);
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
        let submessage = InfoDestinationSubmessageRead::new(&[
            0x0e, 0b_0000_0001, 12, 0, // Submessage header
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
        ]);

        let expected_guid_prefix = GUIDPREFIX_UNKNOWN;
        assert_eq!(expected_guid_prefix, submessage.guid_prefix());
    }
}
