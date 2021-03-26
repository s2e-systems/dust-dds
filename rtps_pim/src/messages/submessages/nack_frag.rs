use crate::messages::{self, submessage_elements, Submessage};

pub trait NackFrag: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::PSM>,
        writer_id: submessage_elements::EntityId<Self::PSM>,
        writer_sn: submessage_elements::SequenceNumber<Self::PSM>,
        fragment_number_state: submessage_elements::FragmentNumberSet<Self::PSM>,
        count: submessage_elements::Count<Self::PSM>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::PSM>;
    fn fragment_number_state(&self) -> &submessage_elements::FragmentNumberSet<Self::PSM>;
    fn count(&self) -> &submessage_elements::Count<Self::PSM>;
}
