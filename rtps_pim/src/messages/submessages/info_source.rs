use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};
pub trait InfoSource<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn protocol_version(&self) -> submessage_elements::ProtocolVersion<PSM>;
    fn vendor_id(&self) -> submessage_elements::VendorId<PSM>;
    fn guid_prefix(&self) -> submessage_elements::GuidPrefix<PSM>;
}
