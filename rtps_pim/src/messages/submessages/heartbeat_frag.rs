use crate::{messages::submessage_elements, PIM};

pub trait HeartbeatFrag<PSM: PIM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn last_fragment_num(&self) -> submessage_elements::FragmentNumber<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
}
