use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait HeartbeatFrag: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber;
    type FragmentNumber: submessage_elements::FragmentNumber;
    type Count: submessage_elements::Count;

    fn new(
        endianness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        writer_sn: Self::SequenceNumber,
        last_fragment_num: Self::FragmentNumber,
        count: Self::Count,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn last_fragment_num(&self) -> &Self::FragmentNumber;
    fn count(&self) -> &Self::Count;
}
