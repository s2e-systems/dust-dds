use crate::{messages::submessage_elements, PIM};

pub struct AckNack<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub final_flag: PSM::SubmessageFlag,
    pub reader_id: submessage_elements::EntityId<PSM>,
    pub writer_id: submessage_elements::EntityId<PSM>,
    pub reader_sn_state: submessage_elements::SequenceNumberSet<PSM>,
    pub count: submessage_elements::Count<PSM>,
}
