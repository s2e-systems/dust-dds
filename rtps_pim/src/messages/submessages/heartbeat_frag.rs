use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait HeartbeatFrag<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn last_fragment_num(&self) -> submessage_elements::FragmentNumber<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
}
