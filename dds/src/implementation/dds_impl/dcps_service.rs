use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread::JoinHandle,
};

use socket2::Socket;

use crate::{
    implementation::{
        rtps::types::{LocatorAddress, LocatorPort},
        rtps_udp_psm::udp_transport::UdpTransport,
        utils::{condvar::DdsCondvar, shared_object::DdsShared},
    },
    infrastructure::{error::DdsResult, time::Duration},
};

use super::domain_participant_impl::DomainParticipantImpl;

pub fn get_multicast_socket(
    multicast_address: LocatorAddress,
    port: LocatorPort,
) -> std::io::Result<UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, <u32>::from(port) as u16));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;

    //socket.set_nonblocking(true).ok()?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;

    socket.bind(&socket_addr.into())?;
    let multicast_addr_bytes: [u8; 16] = multicast_address.into();
    let addr = Ipv4Addr::new(
        multicast_addr_bytes[12],
        multicast_addr_bytes[13],
        multicast_addr_bytes[14],
        multicast_addr_bytes[15],
    );
    socket.join_multicast_v4(&addr, &Ipv4Addr::UNSPECIFIED)?;
    socket.set_multicast_loop_v4(true)?;

    Ok(socket.into())
}

pub fn get_unicast_socket(port: LocatorPort) -> std::io::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddr::from((
        Ipv4Addr::UNSPECIFIED,
        <u32>::from(port) as u16,
    )))?;
    socket.set_nonblocking(true)?;

    Ok(socket)
}

pub struct DcpsService {
    participant: DdsShared<DomainParticipantImpl>,
    quit: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
    announcer_condvar: DdsCondvar,
    sedp_condvar: DdsCondvar,
    user_defined_data_send_condvar: DdsCondvar,
}

impl DcpsService {
    pub fn new(participant: DdsShared<DomainParticipantImpl>) -> DdsResult<Self> {
        let quit = Arc::new(AtomicBool::new(false));
        let mut threads = Vec::new();

        // //////////// Notification thread
        {
            let domain_participant = participant.clone();
            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                domain_participant.update_communication_status().ok();
                std::thread::sleep(std::time::Duration::from_millis(50));
            }));
        }

        // //////////// SPDP Communication

        // ////////////// SPDP participant discovery
        {
            let domain_participant = participant.clone();
            let l = domain_participant
                .metatraffic_multicast_locator_list()
                .get(0)
                .unwrap();

            let mut metatraffic_multicast_transport =
                UdpTransport::new(get_multicast_socket(l.address(), l.port()).unwrap());
            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) = metatraffic_multicast_transport
                    .read(Some(std::time::Duration::from_millis(1000)))
                {
                    domain_participant
                        .receive_built_in_data(locator, message)
                        .ok();
                }
            }));
        }

        // //////////// Unicast metatraffic Communication receive
        {
            let domain_participant = participant.clone();
            let l = domain_participant
                .metatraffic_unicast_locator_list()
                .get(0)
                .unwrap();
            let mut metatraffic_unicast_transport =
                UdpTransport::new(get_unicast_socket(l.port()).unwrap());

            let task_quit = quit.clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) =
                    metatraffic_unicast_transport.read(Some(std::time::Duration::from_millis(1000)))
                {
                    domain_participant
                        .receive_built_in_data(locator, message)
                        .ok();
                }
            }));
        }

        // //////////// Unicast metatraffic Communication send
        {
            let domain_participant = participant.clone();
            let socket = UdpSocket::bind("0.0.0.0:0000").unwrap();

            let mut metatraffic_unicast_transport_send = UdpTransport::new(socket);
            let task_quit = quit.clone();
            let sedp_condvar_clone = domain_participant.sedp_condvar().clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }
                let _r = sedp_condvar_clone.wait_timeout(Duration::new(0, 500000000));

                domain_participant.send_built_in_data(&mut metatraffic_unicast_transport_send);
            }));
        }

        // //////////// User-defined Communication receive
        {
            let domain_participant = participant.clone();
            let default_unicast_locator_list = domain_participant
                .default_unicast_locator_list()
                .get(0)
                .unwrap();
            let mut default_unicast_transport =
                UdpTransport::new(get_unicast_socket(default_unicast_locator_list.port()).unwrap());
            let task_quit = quit.clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) =
                    default_unicast_transport.read(Some(std::time::Duration::from_millis(1000)))
                {
                    domain_participant
                        .receive_user_defined_data(locator, message)
                        .ok();
                }
            }));
        }

        // //////////// User-defined Communication send
        {
            let domain_participant = participant.clone();
            let socket = UdpSocket::bind("0.0.0.0:0000").unwrap();
            let mut default_unicast_transport_send = UdpTransport::new(socket);
            let task_quit = quit.clone();
            let user_defined_data_send_condvar_clone =
                domain_participant.user_defined_data_send_condvar().clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                let _r = user_defined_data_send_condvar_clone
                    .wait_timeout(Duration::new(0, 100_000_000));

                domain_participant.send_user_defined_data(&mut default_unicast_transport_send);
            }));
        }

        {
            // //////////// Announce participant
            let domain_participant = participant.clone();
            let task_quit = quit.clone();
            let announcer_condvar_clone = domain_participant.announcer_condvar().clone();
            threads.push(std::thread::spawn(move || {
                // TODO: Temporary solution to ensure tests pass by announcing as soon as possible
                domain_participant.announce_participant().ok();
                loop {
                    if task_quit.load(atomic::Ordering::SeqCst) {
                        break;
                    }

                    let _r = announcer_condvar_clone.wait_timeout(Duration::new(5, 0));

                    match domain_participant.announce_participant() {
                        Ok(_) => (),
                        Err(e) => println!("participant announcement failed: {:?}", e),
                    }
                }
            }));
        }
        let announcer_condvar = participant.announcer_condvar().clone();
        let sedp_condvar = participant.sedp_condvar().clone();
        let user_defined_data_send_condvar = participant.user_defined_data_send_condvar().clone();
        Ok(DcpsService {
            participant,
            quit,
            threads,
            announcer_condvar,
            sedp_condvar,
            user_defined_data_send_condvar,
        })
    }

    pub fn participant(&self) -> &DdsShared<DomainParticipantImpl> {
        &self.participant
    }

    pub fn shutdown_tasks(&mut self) {
        self.quit.store(true, atomic::Ordering::SeqCst);
        self.announcer_condvar.notify_all();
        self.sedp_condvar.notify_all();
        self.user_defined_data_send_condvar.notify_all();

        while let Some(thread) = self.threads.pop() {
            thread.join().unwrap();
        }
    }
}
