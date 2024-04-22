use dust_dds::rtps::messages::overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

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

    let mut buf = [0u8; 65000];
    println!("Starting Dust DDS Telescope");
    loop {
        let received_size = socket.recv(&mut buf).unwrap();
        println!("Received data {received_size}");
        if let Ok(m) = RtpsMessageRead::new(buf[0..received_size].into()) {
            for submessage in m.submessages() {
                if let RtpsSubmessageReadKind::Data(d) = submessage {
                    println!("Received data submessage {:?}", d)
                }
            }

            println!("Received an RTPS message {:?}", m);
        } else {
            println!("Received data not representing an RTPS message")
        }
    }

    // socket.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;
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
