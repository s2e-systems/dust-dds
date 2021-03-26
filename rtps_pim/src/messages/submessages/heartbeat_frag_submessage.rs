use super::{submessage_elements, Submessage};
use crate::{messages, types};

pub trait HeartbeatFrag: Submessage {
    type EntityId: types::EntityId;
    type SequenceNumber: types::SequenceNumber;
    type FragmentNumber: messages::types::FragmentNumber;
    type Count: messages::types::Count;

    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        writer_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        last_fragment_num: submessage_elements::FragmentNumber<Self::FragmentNumber>,
        count: submessage_elements::Count<Self::Count>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId>;
    fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber>;
    fn last_fragment_num(&self) -> &submessage_elements::FragmentNumber<Self::FragmentNumber>;
    fn count(&self) -> &submessage_elements::Count<Self::Count>;
}
