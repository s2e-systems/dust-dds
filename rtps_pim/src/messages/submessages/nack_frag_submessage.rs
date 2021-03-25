use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait NackFrag: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber;
    type FragmentNumberSet: submessage_elements::FragmentNumberSet;
    type Count: submessage_elements::Count;

    fn new(
        endianness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        writer_sn: Self::SequenceNumber,
        fragment_number_state: Self::FragmentNumberSet,
        count: Self::Count,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn fragment_number_state(&self) -> &Self::FragmentNumberSet;
    fn count(&self) -> &Self::Count;
}
