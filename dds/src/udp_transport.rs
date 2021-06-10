use rust_dds_rtps_implementation::transport::Transport;
use rust_rtps_pim::structure::types::LocatorPIM;
use rust_rtps_udp_psm::RtpsUdpPsm;
use rust_serde_cdr::serializer::RtpsMessageSerializer;

pub struct UdpTransport{
    serializer: RtpsMessageSerializer<Vec<u8>>
}

impl UdpTransport {
    pub fn new() -> Self {
        Self {
            serializer: RtpsMessageSerializer{writer: Vec::new()}
        }
    }
}

impl Transport<RtpsUdpPsm> for UdpTransport {
    fn unicast_locator_list(&self) -> &[<RtpsUdpPsm as LocatorPIM>::LocatorType] {
        todo!()
    }
}