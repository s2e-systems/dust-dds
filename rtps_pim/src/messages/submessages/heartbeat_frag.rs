use crate::{messages::submessage_elements, PIM};

pub struct HeartbeatFrag<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub reader_id: submessage_elements::EntityId<PSM>,
    pub writer_id: submessage_elements::EntityId<PSM>,
    pub writer_sn: submessage_elements::SequenceNumber<PSM>,
    pub last_fragment_num: submessage_elements::FragmentNumber<PSM>,
    pub count: submessage_elements::Count<PSM>,
}
