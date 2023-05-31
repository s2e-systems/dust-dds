use crate::implementation::rtps::messages::{
    overall_structure::{SubmessageHeaderRead, SubmessageHeader}, types::SubmessageFlag,
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
pub struct PadSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deserialize_pad() {
        #[rustfmt::skip]
        let submessage = PadSubmessageRead::new(&[
            0x01, 0b_0000_0001, 0, 0, // Submessage header
        ]);
        let expected_endianness_flag = true;
        assert_eq!(expected_endianness_flag, submessage.submessage_header().endianness_flag());
    }
}