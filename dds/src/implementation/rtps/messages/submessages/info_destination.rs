use crate::implementation::rtps::{
    messages::{
        overall_structure::{RtpsMap, SubmessageHeader, SubmessageHeaderRead},
        types::SubmessageFlag,
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
pub struct InfoDestinationSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub guid_prefix: GuidPrefix,
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps::types::GUIDPREFIX_UNKNOWN;

    use super::*;

    #[test]
    fn deserialize_info_destination() {
        #[rustfmt::skip]
        let submessage = InfoDestinationSubmessageRead::new(&[
            0x0c, 0b_0000_0001, 12, 0, // Submessage header
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
            0, 0, 0, 0, //guid_prefix
        ]);

        let expected_guid_prefix = GUIDPREFIX_UNKNOWN;
        assert_eq!(expected_guid_prefix, submessage.guid_prefix());
    }
}
