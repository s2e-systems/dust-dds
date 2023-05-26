use crate::implementation::rtps::{
    messages::{
        overall_structure::SubmessageHeaderRead, types::SubmessageFlag, RtpsMap, SubmessageHeader,
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
