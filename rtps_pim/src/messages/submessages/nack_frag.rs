use crate::{messages::submessage_elements, PIM};

pub trait NackFrag<PSM: PIM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn fragment_number_state(&self) -> submessage_elements::FragmentNumberSet<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
}
