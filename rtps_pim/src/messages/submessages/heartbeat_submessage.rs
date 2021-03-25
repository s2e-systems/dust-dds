use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait Heartbeat: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber;
    type Count: submessage_elements::Count;

    fn new(
        endianness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        final_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        liveliness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        reader_id: Self::EntityId,
        writer_id: Self::EntityId,
        first_sn: Self::SequenceNumber,
        last_sn: Self::SequenceNumber,
        count: Self::Count,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn final_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn liveliness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    // group_info_flag: SubmessageFlag,
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn first_sn(&self) -> &Self::SequenceNumber;
    fn last_sn(&self) -> &Self::SequenceNumber;
    fn count(&self) -> &Self::Count;
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}
