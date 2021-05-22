use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};
pub trait Heartbeat<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn final_flag(&self) -> PSM::SubmessageFlag;
    fn liveliness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn first_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn last_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn count(&self) -> submessage_elements::Count<PSM>;
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}
