use rust_dds_rtps_implementation::transport::{TransportLocator, TransportRead, TransportWrite};
use rust_rtps_pim::{messages::RTPSMessagePIM, structure::types::Locator};
use rust_rtps_udp_psm::psm::RtpsUdpPsm;
use rust_serde_cdr::serializer::RtpsMessageSerializer;

pub struct UdpTransport {
    serializer: RtpsMessageSerializer<Vec<u8>>,
}

impl UdpTransport {
    pub fn new() -> Self {
        Self {
            serializer: RtpsMessageSerializer { writer: Vec::new() },
        }
    }
}

impl TransportWrite<RtpsUdpPsm> for UdpTransport {
    fn write<'a>(
        &mut self,
        _message: &[rust_rtps_pim::messages::submessages::RtpsSubmessageType<'a, RtpsUdpPsm>],
        _destination_locator: &Locator,
    ) where
        RtpsUdpPsm: rust_rtps_pim::messages::submessages::AckNackSubmessagePIM
            + rust_rtps_pim::messages::submessages::DataSubmessagePIM<'a, RtpsUdpPsm>
            + rust_rtps_pim::messages::submessages::DataFragSubmessagePIM<'a>
            + rust_rtps_pim::messages::submessages::GapSubmessagePIM
            + rust_rtps_pim::messages::submessages::HeartbeatSubmessagePIM
            + rust_rtps_pim::messages::submessages::HeartbeatFragSubmessagePIM
            + rust_rtps_pim::messages::submessages::InfoDestinationSubmessagePIM
            + rust_rtps_pim::messages::submessages::InfoReplySubmessagePIM
            + rust_rtps_pim::messages::submessages::InfoSourceSubmessagePIM
            + rust_rtps_pim::messages::submessages::InfoTimestampSubmessagePIM
            + rust_rtps_pim::messages::submessages::NackFragSubmessagePIM
            + rust_rtps_pim::messages::submessages::PadSubmessagePIM,
    {
        todo!()
    }
}

impl TransportRead<RtpsUdpPsm> for UdpTransport {
    fn read<'a>(
        &self,
    ) -> Option<(
        <RtpsUdpPsm as RTPSMessagePIM<'a, RtpsUdpPsm>>::RTPSMessageType,
        Locator,
    )>
    where
        RtpsUdpPsm: RTPSMessagePIM<'a, RtpsUdpPsm>,
    {
        todo!()
    }
}

impl TransportLocator for UdpTransport {
    fn unicast_locator_list(&self) -> &[Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        todo!()
    }
}
