use crate::implementation::rtps::messages::{
    overall_structure::{
        Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
    },
    submessage_elements::SubmessageElement,
    types::SubmessageKind,
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
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
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
impl Submessage for PadSubmessageWrite {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::PAD, &[], octets_to_next_header)
    }

    fn submessage_elements(&self) -> &[SubmessageElement] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::messages::overall_structure::into_bytes_vec;

    #[test]
    fn serialize_pad() {
        let submessage = PadSubmessageWrite::new();
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x01, 0b_0000_0001, 0, 0, // Submessage header
            ]
        );
    }

    #[test]
    fn deserialize_pad() {
        #[rustfmt::skip]
        let submessage = PadSubmessageRead::new(&[
            0x01, 0b_0000_0001, 0, 0, // Submessage header
        ]);
        let expected_endianness_flag = true;
        assert_eq!(
            expected_endianness_flag,
            submessage.submessage_header().endianness_flag()
        );
    }
}
