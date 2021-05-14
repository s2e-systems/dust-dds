use crate::{messages::submessage_elements, PIM};

pub struct NackFrag<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub reader_id: submessage_elements::EntityId<PSM>,
    pub writer_id: submessage_elements::EntityId<PSM>,
    pub writer_sn: submessage_elements::SequenceNumber<PSM>,
    pub fragment_number_state: submessage_elements::FragmentNumberSet<PSM>,
    pub count: submessage_elements::Count<PSM>,
}
