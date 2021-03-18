pub mod memory;

use rust_dds_api::return_type::DDSResult;
use rust_rtps::{messages::RtpsMessage, types::Locator};

pub trait Transport: Send + Sync {
    fn write<'a>(&'a self, message: RtpsMessage<'a>, destination_locator: &Locator);

    // fn read<'a>(&'a self) -> DDSResult<Option<(RtpsMessage<'a>, Locator)>>;

    fn unicast_locator_list(&self) -> &[Locator];

    fn multicast_locator_list(&self) -> &[Locator];
}
