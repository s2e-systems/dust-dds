use crate::{messages::submessage_elements, PIM};
pub struct Heartbeat<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub final_flag: PSM::SubmessageFlag,
    pub liveliness_flag: PSM::SubmessageFlag,
    pub reader_id: submessage_elements::EntityId<PSM>,
    pub writer_id: submessage_elements::EntityId<PSM>,
    pub first_sn: submessage_elements::SequenceNumber<PSM>,
    pub last_sn: submessage_elements::SequenceNumber<PSM>,
    pub count: submessage_elements::Count<PSM>,
    // current_gsn: submessage_elements::SequenceNumber,
    // first_gsn: submessage_elements::SequenceNumber,
    // last_gsn: submessage_elements::SequenceNumber,
    // writer_set: submessage_elements::GroupDigest,
    // secure_writer_set: submessage_elements::GroupDigest,
}
