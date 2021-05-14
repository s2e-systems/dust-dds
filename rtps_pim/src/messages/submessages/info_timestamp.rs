use crate::{messages::submessage_elements, PIM};

pub struct InfoTimestamp<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub invalidate_flag: PSM::SubmessageFlag,
    pub timestamp: submessage_elements::Timestamp<PSM>,
}
