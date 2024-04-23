use dust_dds::{
    data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    rtps::messages::overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
    topic_definition::type_support::DdsDeserialize,
};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    collections::{hash_map::Entry, HashMap},
    net::{Ipv4Addr, SocketAddr, UdpSocket},
};

fn main() {
    let port = 7400;
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .unwrap();

    socket.set_reuse_address(true).unwrap();
    socket.set_nonblocking(false).unwrap();
    socket.bind(&socket_addr.into()).unwrap();

    let socket = UdpSocket::from(socket);

    let multicast_addr = Ipv4Addr::new(239, 255, 0, 1);
    let interface_address_list = get_interface_address_list(None);
    for interface_addr in interface_address_list {
        match interface_addr {
            Addr::V4(a) => {
                let r = socket.join_multicast_v4(&multicast_addr, &a.ip);
                if let Err(e) = r {
                    println!(
                        "Failed to join multicast group on address {} with error {}",
                        a.ip, e
                    )
                }
            }
            Addr::V6(_) => (),
        }
    }

    let mut participant_list = HashMap::new();

    let mut buf = [0u8; 65000];
    println!("Starting Dust DDS Telescope");
    loop {
        let received_size = socket.recv(&mut buf).unwrap();
        if let Ok(m) = RtpsMessageRead::new(buf[0..received_size].into()) {
            for submessage in m.submessages() {
                if let RtpsSubmessageReadKind::Data(d) = submessage {
                    if let Ok(discovered_participant) =
                        SpdpDiscoveredParticipantData::deserialize_data(
                            d.serialized_payload().as_ref(),
                        )
                    {
                        match participant_list
                            .entry(discovered_participant.participant_proxy().guid_prefix())
                        {
                            Entry::Occupied(_) => (),
                            Entry::Vacant(e) => {
                                println!(
                                    "Discovered participant GUID {:?} on domain {:?} with tag {:?}",
                                    discovered_participant.participant_proxy().guid_prefix(),
                                    discovered_participant.participant_proxy().domain_id(),
                                    discovered_participant.participant_proxy().domain_tag(),
                                );
                                println!(
                                    "Participant metattrafic unicast locator list {:?}",
                                    discovered_participant
                                        .participant_proxy()
                                        .metatraffic_unicast_locator_list()
                                );
                                println!(
                                    "Participant metattrafic multicast locator list {:?}",
                                    discovered_participant
                                        .participant_proxy()
                                        .metatraffic_multicast_locator_list()
                                );
                                println!(
                                    "Participant default unicast locator list {:?}",
                                    discovered_participant
                                        .participant_proxy()
                                        .default_unicast_locator_list()
                                );
                                println!(
                                    "Participant default multicast locator list {:?}",
                                    discovered_participant
                                        .participant_proxy()
                                        .default_multicast_locator_list()
                                );
                                e.insert(discovered_participant);
                                println!("\n\n")
                            }
                        }
                    }
                }
            }
        } else {
            println!("Received data not representing an RTPS message");
        }
    }
}

fn get_interface_address_list(interface_name: Option<&String>) -> Vec<Addr> {
    NetworkInterface::show()
        .expect("Could not scan interfaces")
        .into_iter()
        .filter(|x| {
            if let Some(if_name) = interface_name {
                &x.name == if_name
            } else {
                true
            }
        })
        .flat_map(|i| {
            i.addr.into_iter().filter(|a| match a {
                #[rustfmt::skip]
                Addr::V4(v4) if !v4.ip.is_loopback() => true,
                _ => false,
            })
        })
        .collect()
}
