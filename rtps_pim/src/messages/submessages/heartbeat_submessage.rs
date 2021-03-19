use super::submessage_elements;
use super::Submessage;
use super::SubmessageFlag;

pub trait Heartbeat: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber: submessage_elements::SequenceNumber;
    type Count: submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn liveliness_flag(&self) -> SubmessageFlag;
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
