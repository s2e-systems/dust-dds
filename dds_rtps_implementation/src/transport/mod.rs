use rust_rtps_pim::{messages::RTPSMessagePIM, structure::types::LocatorPIM};

pub trait Transport<PSM>: Send + Sync
where
    PSM: LocatorPIM,
{
    fn write<'a>(
        &mut self,
        message: &<PSM as RTPSMessagePIM<'a, PSM>>::RTPSMessageType,
        destination_locator: &PSM::LocatorType,
    ) where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn read<'a>(&'a self) -> Option<(PSM::RTPSMessageType, PSM::LocatorType)>
    where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn unicast_locator_list(&self) -> &[PSM::LocatorType];

    fn multicast_locator_list(&self) -> &[PSM::LocatorType];
}
