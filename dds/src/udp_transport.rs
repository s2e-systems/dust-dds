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
        _message: &<RtpsUdpPsm as RTPSMessagePIM<'a>>::RTPSMessageType,
        _destination_locator: &Locator,
    ) where
        RtpsUdpPsm: RTPSMessagePIM<'a>,
    {
        todo!()
    }
}

impl TransportRead<RtpsUdpPsm> for UdpTransport {
    fn read<'a>(&self) -> Option<(<RtpsUdpPsm as RTPSMessagePIM<'a>>::RTPSMessageType, Locator)>
    where
        RtpsUdpPsm: RTPSMessagePIM<'a>,
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
