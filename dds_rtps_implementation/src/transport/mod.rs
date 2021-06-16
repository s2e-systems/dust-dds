use rust_rtps_pim::{messages::{RTPSMessage, RTPSMessagePIM, RtpsMessageHeaderPIM, types::{ProtocolIdPIM, SubmessageKindPIM}}, structure::types::{GuidPrefixPIM, LocatorPIM, ProtocolVersionPIM, VendorIdPIM}};

pub trait Transport<
    PSM: LocatorPIM
        + ProtocolIdPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + for<'a> RtpsMessageHeaderPIM<'a, PSM>
        + SubmessageKindPIM,
>: Send + Sync
{
    fn write<'a>(&mut self, message: &<<PSM as RTPSMessagePIM<'a, PSM>>::RTPSMessageType as RTPSMessage<'a, PSM>>::Constructed, destination_locator: &PSM::LocatorType)
    where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn read<'a>(&'a self) -> Option<(PSM::RTPSMessageType, PSM::LocatorType)>
    where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn unicast_locator_list(&self) -> &[PSM::LocatorType];

    fn multicast_locator_list(&self) -> &[PSM::LocatorType];
}
