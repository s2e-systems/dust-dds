use crate::{messages::submessage_elements, PIM};
pub struct InfoSource<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub protocol_version: submessage_elements::ProtocolVersion<PSM>,
    pub vendor_id: submessage_elements::VendorId<PSM>,
    pub guid_prefix: submessage_elements::GuidPrefix<PSM>,
}
