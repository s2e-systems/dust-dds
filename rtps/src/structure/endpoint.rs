use crate::types::{Locator,GuidPrefix};
use crate::messages::RtpsSubmessage;

pub trait RtpsEndpoint{
    fn try_push_message(&self, src_locator: Locator, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>);
}