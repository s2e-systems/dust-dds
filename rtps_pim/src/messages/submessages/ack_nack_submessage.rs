use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait AckNack: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumberSet: submessage_elements::SequenceNumberSet;
    type Count: submessage_elements::Count;

    fn new(
        endianness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        final_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        reader_id: Self::EntityId,
        writer: Self::EntityId,
        reader_sn_state: Self::SequenceNumberSet,
        count: Self::Count,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn final_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn reader_sn_state(&self) -> &Self::SequenceNumberSet;
    fn count(&self) -> &Self::Count;
}