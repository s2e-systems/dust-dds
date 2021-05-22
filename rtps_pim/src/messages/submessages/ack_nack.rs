use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait AckNack<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn final_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn reader_sn_state(&self) -> submessage_elements::SequenceNumberSet<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
}
