use crate::implementation::rtps::messages::{
    overall_structure::SubmessageHeaderRead,
    types::{SubmessageFlag, Time, TIME_INVALID},
    RtpsMap, SubmessageHeader,
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
