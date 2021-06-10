use rust_rtps_pim::{
    messages::{
        types::{ProtocolIdPIM, SubmessageFlagPIM, SubmessageKindPIM},
        RTPSMessagePIM,
    },
    structure::types::{GuidPrefixPIM, LocatorPIM, ProtocolVersionPIM, VendorIdPIM},
};

pub trait Transport<
    PSM: LocatorPIM
        + ProtocolIdPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + SubmessageFlagPIM
        + SubmessageKindPIM
>: Send + Sync
{
    fn write<'a>(&self, message: &PSM::RTPSMessageType, destination_locator: &PSM::LocatorType) where PSM: RTPSMessagePIM<'a, PSM>;

    // fn read<'a>(&'a self) -> DDSResult<Option<(RtpsMessage<'a>, Locator)>>;

    fn unicast_locator_list(&self) -> &[PSM::LocatorType];

    // fn multicast_locator_list(&self) -> &[PSM::LocatorType];
}
