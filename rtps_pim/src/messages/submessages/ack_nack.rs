use crate::messages::{self, submessage_elements, Submessage};

pub trait AckNack: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        final_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::PSM>,
        writer_id: submessage_elements::EntityId<Self::PSM>,
        reader_sn_state: submessage_elements::SequenceNumberSet<Self::PSM>,
        count: submessage_elements::Count<Self::PSM>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn final_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn reader_sn_state(&self) -> &submessage_elements::SequenceNumberSet<Self::PSM>;
    fn count(&self) -> &submessage_elements::Count<Self::PSM>;
}
