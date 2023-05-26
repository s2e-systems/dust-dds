use crate::implementation::rtps::{
    messages::{
        overall_structure::SubmessageHeaderRead, submessage_elements::SequenceNumberSet,
        types::SubmessageFlag, RtpsMap, SubmessageHeader,
    },
    types::{EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for GapSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> GapSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn reader_id(&self) -> EntityId {
        self.map(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.map(&self.data[8..])
    }

    pub fn gap_start(&self) -> SequenceNumber {
        self.map(&self.data[12..])
    }

    pub fn gap_list(&self) -> SequenceNumberSet {
        self.map(&self.data[20..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub gap_start: SequenceNumber,
    pub gap_list: SequenceNumberSet,
}
