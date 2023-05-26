use crate::implementation::rtps::{
    messages::{
        overall_structure::SubmessageHeaderRead,
        types::{FragmentNumber, SubmessageFlag},
        RtpsMap, SubmessageHeader,
    },
    types::{Count, EntityId, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for HeartbeatFragSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> HeartbeatFragSubmessageRead<'a> {
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

    pub fn last_fragment_num(&self) -> FragmentNumber {
        self.map(&self.data[20..])
    }

    pub fn count(&self) -> Count {
        self.map(&self.data[24..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub last_fragment_num: FragmentNumber,
    pub count: Count,
}
