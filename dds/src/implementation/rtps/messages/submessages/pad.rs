use crate::implementation::rtps::messages::{
    overall_structure::SubmessageHeaderRead, types::SubmessageFlag, SubmessageHeader,
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
