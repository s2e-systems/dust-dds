use rust_rtps_pim::structure::types::{LocatorPIM};

// pub mod memory;


pub trait Transport<PSM: LocatorPIM>: Send + Sync {
    // fn write<'a>(&'a self, message: RtpsMessage<'a>, destination_locator: &Locator);

    // fn read<'a>(&'a self) -> DDSResult<Option<(RtpsMessage<'a>, Locator)>>;

    fn unicast_locator_list(&self) -> &[PSM::LocatorType];

    // fn multicast_locator_list(&self) -> &[PSM::LocatorType];
}

