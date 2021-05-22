use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait Gap<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn new(
        endianness_flag: PSM::SubmessageFlag,
        reader_id: PSM::EntityId,
        writer_id: PSM::EntityId,
        gap_start: PSM::SequenceNumber,
        gap_list: PSM::SequenceNumberVector,
    ) -> Self;
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn gap_start(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn gap_list(&self) -> submessage_elements::SequenceNumberSet<PSM>;
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}
