use super::{submessage_elements, Submessage};
use crate::{messages, types};

pub trait Heartbeat: Submessage {
    type EntityId: types::EntityId;
    type SequenceNumber: types::SequenceNumber;
    type Count: messages::types::Count;

    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        final_flag: <Self as Submessage>::SubmessageFlag,
        liveliness_flag: <Self as Submessage>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        first_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        last_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        count: submessage_elements::Count<Self::Count>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn final_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn liveliness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    // group_info_flag: SubmessageFlag,
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn first_sn(&self) -> &Self::SequenceNumber;
    fn last_sn(&self) -> &Self::SequenceNumber;
    fn count(&self) -> &Self::Count;
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}
