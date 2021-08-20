use std::{
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket},
};

use rust_dds_rtps_implementation::{
    dds_impl::{publisher_impl::PublisherStorage, subscriber_impl::SubscriberStorage},
    utils::{
        message_receiver::MessageReceiver,
        shared_object::RtpsShared,
        transport::{TransportMessage, TransportRead, TransportWrite},
    },
};
use rust_rtps_pim::{
    messages::{submessage_elements::Parameter, submessages::RtpsSubmessageType, RtpsMessage},
    structure::{
        types::{LOCATOR_KIND_UDPv4, LOCATOR_KIND_UDPv6, Locator, SequenceNumber},
        RtpsEntity, RtpsParticipant,
    },
};
use rust_rtps_udp_psm::{deserialize::from_bytes_le, serialize::to_writer_le};

const BUFFER_SIZE: usize = 32000;
pub struct UdpTransport {
    socket: UdpSocket,
    receive_buffer: [u8; BUFFER_SIZE],
}

pub fn send_udp_data(
    rtps_participant: &(impl RtpsParticipant + RtpsEntity),
    publishers: &[RtpsShared<PublisherStorage>],
    transport: &mut UdpTransport,
) {
    for publisher in publishers {
        let publisher_lock = publisher.lock();
        for data_writer in publisher_lock.data_writer_storage_list() {
            let mut data_writer_lock = data_writer.lock();
            rust_dds_rtps_implementation::utils::message_sender::send_data(
                rtps_participant,
                &mut data_writer_lock.rtps_data_writer_mut(),
                transport,
            );
        }
    }
}

pub fn receive_udp_data(
    rtps_participant: &(impl RtpsParticipant + RtpsEntity),
    subscribers: &[RtpsShared<SubscriberStorage>],
    transport: &mut UdpTransport,
) {
    if let Some((source_locator, message)) = transport.read() {
        MessageReceiver::new().process_message(
            *rtps_participant.guid().prefix(),
            subscribers,
            source_locator,
            &message,
        );
    }
}

struct UdpLocator(Locator);

impl ToSocketAddrs for UdpLocator {
    type Iter = std::option::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        #[allow(non_upper_case_globals)]
        match self.0.kind() {
            &LOCATOR_KIND_UDPv4 => {
                let locator_address = self.0.address();
                let address = SocketAddrV4::new(
                    Ipv4Addr::new(
                        locator_address[12],
                        locator_address[13],
                        locator_address[14],
                        locator_address[15],
                    ),
                    *self.0.port() as u16,
                );
                Ok(Some(SocketAddr::V4(address)).into_iter())
            }
            &LOCATOR_KIND_UDPv6 => todo!(),
            _ => Err(std::io::ErrorKind::InvalidInput.into()),
        }
    }
}

impl From<SocketAddr> for UdpLocator {
    fn from(socket_addr: SocketAddr) -> Self {
        match socket_addr {
            SocketAddr::V4(socket_addr) => {
                let port = socket_addr.port() as u32;
                let address = socket_addr.ip().octets();
                let locator = Locator::new(
                    LOCATOR_KIND_UDPv4,
                    port,
                    [
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, address[0], address[1], address[2],
                        address[3],
                    ],
                );
                UdpLocator(locator)
            }
            SocketAddr::V6(_) => todo!(),
        }
    }
}

impl UdpTransport {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            receive_buffer: [0; BUFFER_SIZE],
        }
    }
}

impl<'a> TransportWrite for UdpTransport {
    fn write(&mut self, message: &TransportMessage, destination_locator: &Locator) {
        let mut writer = Vec::<u8>::new();
        to_writer_le(message, &mut writer).unwrap();
        self.socket
            .send_to(writer.as_slice(), UdpLocator(*destination_locator))
            .expect(&format!(
                "Error sending message to {:?}",
                destination_locator
            ));
    }
}

impl<'a> TransportRead for UdpTransport {
    fn read(&mut self) -> Option<(Locator, TransportMessage)> {
        match self.socket.recv_from(&mut self.receive_buffer) {
            Ok((bytes, source_address)) => {
                if bytes > 0 {
                    let message = from_bytes_le(&self.receive_buffer[0..bytes])
                        .expect("Failed to deserialize");
                    let udp_locator: UdpLocator = source_address.into();
                    Some((udp_locator.0, message))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    use rust_rtps_pim::{
        messages::RtpsMessageHeader,
        structure::types::{
            LOCATOR_KIND_UDPv4, Locator, LOCATOR_INVALID, PROTOCOLVERSION_2_4, VENDOR_ID_S2E,
        },
    };

    use crate::udp_transport::UdpTransport;

    #[test]
    fn udpv4_locator_conversion_address1() {
        let locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        );

        let mut socket_addrs = UdpLocator(locator).to_socket_addrs().unwrap().into_iter();
        let expected_socket_addr = SocketAddr::from_str("127.0.0.1:7400").unwrap();
        assert_eq!(socket_addrs.next(), Some(expected_socket_addr));
    }

    #[test]
    fn udpv4_locator_conversion_address2() {
        let locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            7500,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 25],
        );

        let mut socket_addrs = UdpLocator(locator).to_socket_addrs().unwrap().into_iter();
        let expected_socket_addr = SocketAddr::from_str("192.168.1.25:7500").unwrap();
        assert_eq!(socket_addrs.next(), Some(expected_socket_addr));
    }

    #[test]
    fn locator_conversion_invalid_locator() {
        assert!(UdpLocator(LOCATOR_INVALID).to_socket_addrs().is_err())
    }

    #[test]
    fn socket_addr_to_locator_conversion() {
        let socket_addr = SocketAddr::from_str("127.0.0.1:7400").unwrap();
        let locator = UdpLocator::from(socket_addr).0;
        assert_eq!(locator.kind(), &LOCATOR_KIND_UDPv4);
        assert_eq!(locator.port(), &7400);
        assert_eq!(
            locator.address(),
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1]
        );
    }

    #[test]
    fn multicast_write() {
        let socket_port = 17400;
        let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], socket_port))).unwrap();
        socket
            .join_multicast_v4(&Ipv4Addr::new(239, 255, 0, 1), &Ipv4Addr::new(127, 0, 0, 1))
            .unwrap();
        let mut transport = UdpTransport::new(socket);
        let header = RtpsMessageHeader {
            protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
            version: PROTOCOLVERSION_2_4,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: [3; 12],
        };
        let destination_locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            socket_port as u32,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        );
        let message1 = RtpsMessage {
            header,
            submessages: vec![],
        };

        transport.write(&message1, &destination_locator);
        let (_locator, received_message1) = transport.read().unwrap();
        assert_eq!(message1, received_message1);
    }

    // #[test]
    // fn roundtrip() {
    //     let header = RtpsMessageHeader {
    //         protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
    //         version: PROTOCOLVERSION_2_4,
    //         vendor_id: VENDOR_ID_S2E,
    //         guid_prefix: [3; 12],
    //     };

    //     let socket_port = 17405;
    //     let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], socket_port))).unwrap();
    //     let mut transport = UdpTransport::new(socket);
    //     let destination_locator = Locator::new(
    //         LOCATOR_KIND_UDPv4,
    //         socket_port as u32,
    //         [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
    //     );

    //     let message1: RtpsMessageUdp = RtpsMessageUdp::new(&header, vec![]);
    //     transport.write(&message1, &destination_locator);
    //     let (_locator, received_message1) = transport.read().unwrap();
    //     assert_eq!(message1, received_message1);

    //     let endianness_flag = true;
    //     let inline_qos_flag = false;
    //     let data_flag = false;
    //     let key_flag = false;
    //     let non_standard_payload_flag = false;
    //     let reader_id = EntityIdUdp {
    //         entity_key: [1, 2, 3],
    //         entity_kind: 4,
    //     };
    //     let writer_id = EntityIdUdp {
    //         entity_key: [6, 7, 8],
    //         entity_kind: 9,
    //     };
    //     let writer_sn = SequenceNumberUdp::new(&5);
    //     let inline_qos = ParameterListUdp {
    //         parameter: vec![].into(),
    //     };
    //     let data = [];
    //     let serialized_payload = SerializedDataUdp(data[..].into());
    //     let submessage = DataSubmessage{
    //         endianness_flag,
    //         inline_qos_flag,
    //         data_flag,
    //         key_flag,
    //         non_standard_payload_flag,
    //         reader_id,
    //         writer_id,
    //         writer_sn,
    //         inline_qos,
    //         serialized_payload,
    //     };
    //     let message2: RtpsMessageUdp =
    //         RtpsMessageUdp::new(&header, vec![RtpsSubmessageType::Data(submessage)]);
    //     transport.write(&message2, &destination_locator);
    //     let (_locator, received_message2) = transport.read().unwrap();
    //     assert_eq!(message2, received_message2);
    // }
}
