use super::{submessage_elements, Submessage};
use crate::{messages, types};

pub trait AckNack: Submessage {
    type EntityId: types::EntityId;
    type SequenceNumber: types::SequenceNumber;
    type SequenceNumberList: IntoIterator<Item = Self::SequenceNumber>;
    type Count: messages::types::Count;

    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        final_flag: <Self as Submessage>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer: submessage_elements::EntityId<Self::EntityId>,
        reader_sn_state: submessage_elements::SequenceNumberSet<Self::SequenceNumber, Self::SequenceNumberList>,
        count: submessage_elements::Count<Self::Count>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn final_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId>;
    fn reader_sn_state(
        &self,
    ) -> &submessage_elements::SequenceNumberSet<Self::SequenceNumber, Self::SequenceNumberList>;
    fn count(&self) -> &submessage_elements::Count<Self::Count>;
}
