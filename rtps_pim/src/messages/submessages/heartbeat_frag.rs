use crate::messages::{self, submessage_elements, Submessage};

pub trait HeartbeatFrag: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::PSM>,
        writer_id: submessage_elements::EntityId<Self::PSM>,
        writer_sn: submessage_elements::SequenceNumber<Self::PSM>,
        last_fragment_num: submessage_elements::FragmentNumber<Self::PSM>,
        count: submessage_elements::Count<Self::PSM>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::PSM>;
    fn last_fragment_num(&self) -> &submessage_elements::FragmentNumber<Self::PSM>;
    fn count(&self) -> &submessage_elements::Count<Self::PSM>;
}
