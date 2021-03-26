use crate::messages::{self, submessage_elements, Submessage};
pub trait Heartbeat: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        final_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        liveliness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::PSM>,
        writer_id: submessage_elements::EntityId<Self::PSM>,
        first_sn: submessage_elements::SequenceNumber<Self::PSM>,
        last_sn: submessage_elements::SequenceNumber<Self::PSM>,
        count: submessage_elements::Count<Self::PSM>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn final_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn liveliness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    // group_info_flag: SubmessageFlag,
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn first_sn(&self) -> &submessage_elements::SequenceNumber<Self::PSM>;
    fn last_sn(&self) -> &submessage_elements::SequenceNumber<Self::PSM>;
    fn count(&self) -> &submessage_elements::Count<Self::PSM>;
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}
