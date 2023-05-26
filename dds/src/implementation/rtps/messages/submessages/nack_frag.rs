use crate::implementation::rtps::{
    messages::{
        overall_structure::SubmessageHeaderRead, submessage_elements::FragmentNumberSet,
        types::SubmessageFlag, RtpsMap, SubmessageHeader,
    },
    types::{Count, EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for NackFragSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> NackFragSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn reader_id(&self) -> EntityId {
        self.map(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.map(&self.data[8..])
    }

    pub fn writer_sn(&self) -> SequenceNumber {
        self.map(&self.data[12..])
    }

    pub fn fragment_number_state(&self) -> FragmentNumberSet {
        self.map(&self.data[20..])
    }

    pub fn count(&self) -> Count {
        self.map(&self.data[self.data.len() - 4..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_number_state: FragmentNumberSet,
    pub count: Count,
}
