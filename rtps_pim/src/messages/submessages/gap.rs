use crate::{messages::{self, submessage_elements, Submessage}, structure};

pub trait Gap: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        reader_id: <<Self::PSM as structure::Types>::Guid as structure::types::Guid>::EntityId,
        writer_id: <<Self::PSM as structure::Types>::Guid as structure::types::Guid>::EntityId,
        gap_start: <Self::PSM as structure::Types>::SequenceNumber,
        gap_list:  <Self::PSM as structure::Types>::SequenceNumberVector,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    // group_info_flag: SubmessageFlag,
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn gap_start(&self) -> &submessage_elements::SequenceNumber<Self::PSM>;
    fn gap_list(&self) -> &submessage_elements::SequenceNumberSet<Self::PSM>;
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}
