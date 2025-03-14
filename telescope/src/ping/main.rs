use dust_dds::{
    rtps::{
        messages::{
            overall_structure::{RtpsMessageHeader, RtpsMessageWrite},
            submessages::ping::PingSubmessage,
        },
        types::{PROTOCOLVERSION_2_4, VENDOR_ID_S2E},
    },
    transport::types::GUIDPREFIX_UNKNOWN,
};
use std::{net::UdpSocket, time::Duration};

pub fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    let rtps_message_header =
        RtpsMessageHeader::new(PROTOCOLVERSION_2_4, VENDOR_ID_S2E, GUIDPREFIX_UNKNOWN);
    for sequence_number in 0..4 {
        let ping_submessage = PingSubmessage::new(sequence_number);
        let rtps_message =
            RtpsMessageWrite::new(&rtps_message_header, &[Box::new(ping_submessage)]);
        socket
            .send_to(rtps_message.buffer(), "192.168.1.185:7400")
            .unwrap();
    }
}
