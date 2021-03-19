use super::submessage_elements;
use super::Submessage;
use super::SubmessageFlag;

pub trait NackFrag: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber: submessage_elements::SequenceNumber;
    type FragmentNumberSet: submessage_elements::FragmentNumberSet;
    type Count: submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn fragment_number_state(&self) -> &Self::FragmentNumberSet;
    fn count(&self) -> &Self::Count;
}
