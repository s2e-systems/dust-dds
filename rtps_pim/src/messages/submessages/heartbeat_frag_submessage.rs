use super::submessage_elements;
use super::Submessage;
use super::SubmessageFlag;

pub trait HeartbeatFrag: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber: submessage_elements::SequenceNumber;
    type FragmentNumber: submessage_elements::FragmentNumber;
    type Count: submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> Self::EntityId;
    fn writer_id(&self) -> Self::EntityId;
    fn writer_sn(&self) -> Self::SequenceNumber;
    fn last_fragment_num(&self) -> Self::FragmentNumber;
    fn count(&self) -> Self::Count;
}
