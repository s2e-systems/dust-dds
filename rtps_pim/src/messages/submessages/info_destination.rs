use crate::{messages::submessage_elements, PIM};

pub struct InfoDestination<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub guid_prefix: submessage_elements::GuidPrefix<PSM>,
}
