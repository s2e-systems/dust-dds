use rust_rtps_pim::{messages::RTPSMessagePIM, structure::types::Locator};

pub trait Transport<PSM>: Send + Sync
{
    fn write<'a>(
        &mut self,
        message: &<PSM as RTPSMessagePIM<'a, PSM>>::RTPSMessageType,
        destination_locator: &Locator,
    ) where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn read<'a>(&'a self) -> Option<(PSM::RTPSMessageType, Locator)>
    where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn unicast_locator_list(&self) -> &[Locator];

    fn multicast_locator_list(&self) -> &[Locator];
}
