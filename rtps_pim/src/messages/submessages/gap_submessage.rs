use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait Gap: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber;
    type SequenceNumberSet: submessage_elements::SequenceNumberSet;

    fn new(
        endianness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        gap_start: Self::SequenceNumber,
        gap_list: Self::SequenceNumberSet,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    // group_info_flag: SubmessageFlag,
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn gap_start(&self) -> &Self::SequenceNumber;
    fn gap_list(&self) -> &Self::SequenceNumberSet;
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}
