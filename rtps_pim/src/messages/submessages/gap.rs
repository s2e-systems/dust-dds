use super::{submessage_elements, Submessage};
use crate::types;

pub trait Gap: Submessage {
    type EntityId: types::EntityId;
    type SequenceNumber: types::SequenceNumber;
    type SequenceNumberList: IntoIterator<Item = Self::SequenceNumber>;

    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        gap_start: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        gap_list: submessage_elements::SequenceNumberSet<
            Self::SequenceNumber,
            Self::SequenceNumberList,
        >,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    // group_info_flag: SubmessageFlag,
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId>;
    fn gap_start(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber>;
    fn gap_list(
        &self,
    ) -> &submessage_elements::SequenceNumberSet<Self::SequenceNumber, Self::SequenceNumberList>;
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}
