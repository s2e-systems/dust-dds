use crate::{
    implementation::rtps::messages::{
        overall_structure::{
            Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
        },
        submessage_elements::SubmessageElement,
        types::SubmessageKind,
    },
    infrastructure::error::{DdsError, DdsResult},
};

#[derive(Debug, PartialEq, Eq)]
pub struct PadSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for PadSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> PadSubmessageRead<'a> {
    pub fn from_bytes(data: &'a [u8]) -> DdsResult<Self> {
        if data.len() >= 4 {
            Ok(Self { data })
        } else {
            Err(DdsError::Error("".to_string()))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PadSubmessageWrite {}

impl PadSubmessageWrite {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for PadSubmessageWrite {
    fn default() -> Self {
        Self::new()
    }
}
impl<'a> Submessage<'a> for PadSubmessageWrite {
    type SubmessageList = &'a [SubmessageElement<'a>];

    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::PAD, &[], octets_to_next_header)
    }

    fn submessage_elements(&self) -> Self::SubmessageList {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::messages::overall_structure::{
        into_bytes_vec, RtpsSubmessageWriteKind,
    };

    #[test]
    fn serialize_pad() {
        let submessage = RtpsSubmessageWriteKind::Pad(PadSubmessageWrite::new());
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x01, 0b_0000_0001, 0, 0, // Submessage header
            ]
        );
    }

    #[test]
    fn deserialize_pad() {
        #[rustfmt::skip]
        let submessage = PadSubmessageRead::from_bytes(&[
            0x01, 0b_0000_0001, 0, 0, // Submessage header
        ]).unwrap();
        let expected_endianness_flag = true;
        assert_eq!(
            expected_endianness_flag,
            submessage.submessage_header().flags()[0]
        );
    }
}
