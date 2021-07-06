use rust_rtps_pim::{messages::RTPSMessagePIM, structure::types::Locator};

pub trait TransportWrite<PSM> {
    fn write<'a>(&mut self, message: &PSM::RTPSMessageType, destination_locator: &Locator)
    where
        PSM: RTPSMessagePIM<'a, PSM>;
}

pub trait TransportRead<PSM> {
    fn read<'a>(&'a self) -> Option<(PSM::RTPSMessageType, Locator)>
    where
        PSM: RTPSMessagePIM<'a, PSM>;
}

pub trait TransportLocator {
    fn unicast_locator_list(&self) -> &[Locator];

    fn multicast_locator_list(&self) -> &[Locator];
}
