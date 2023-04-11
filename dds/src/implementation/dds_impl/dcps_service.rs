use std::{
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    sync::{
        atomic::{self, AtomicBool},
        mpsc::{Receiver, SyncSender},
        Arc,
    },
    thread::JoinHandle,
};

use crate::{
    implementation::{
        rtps_udp_psm::udp_transport::UdpTransport,
        utils::{condvar::DdsCondvar, shared_object::DdsShared},
    },
    infrastructure::{error::DdsResult, time::Duration},
};

use super::domain_participant_impl::{AnnounceKind, DomainParticipantImpl};

pub struct DcpsService {
    participant: DdsShared<DomainParticipantImpl>,
    quit: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
    sedp_condvar: DdsCondvar,
    user_defined_data_send_condvar: DdsCondvar,
    sender_socket: UdpSocket,
    announce_sender: SyncSender<AnnounceKind>,
}

impl DcpsService {
    pub fn new(
        participant: DdsShared<DomainParticipantImpl>,
        mut metatraffic_multicast_transport: UdpTransport,
        mut metatraffic_unicast_transport: UdpTransport,
        mut default_unicast_transport: UdpTransport,
        announce_sender: SyncSender<AnnounceKind>,
        announce_receiver: Receiver<AnnounceKind>,
    ) -> DdsResult<Self> {
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

            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) = metatraffic_multicast_transport.read() {
                    domain_participant
                        .receive_built_in_data(locator, message)
                        .ok();
                }
            }));
        }

        //  ////////////// Entity announcer thread
        {
            let domain_participant = participant.clone();

            let task_quit = quit.clone();

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Ok(r) = announce_receiver.recv() {
                    domain_participant.announce(r).ok();
                }
            }));
        }

        // //////////// Unicast metatraffic Communication receive
        {
            let domain_participant = participant.clone();

            let task_quit = quit.clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) = metatraffic_unicast_transport.read() {
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

                domain_participant
                    .send_built_in_data(&mut metatraffic_unicast_transport_send)
                    .unwrap();
            }));
        }

        // //////////// User-defined Communication receive
        {
            let domain_participant = participant.clone();
            let task_quit = quit.clone();
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                if let Some((locator, message)) = default_unicast_transport.read() {

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

                domain_participant
                    .send_user_defined_data(&mut default_unicast_transport_send)
                    .unwrap();
            }));
        }

        let sender_socket = UdpSocket::bind("0.0.0.0:0000").unwrap();

        let sedp_condvar = participant.sedp_condvar().clone();
        let user_defined_data_send_condvar = participant.user_defined_data_send_condvar().clone();
        Ok(DcpsService {
            participant,
            quit,
            threads,
            sedp_condvar,
            user_defined_data_send_condvar,
            sender_socket,
            announce_sender,
        })
    }

    pub fn participant(&self) -> &DdsShared<DomainParticipantImpl> {
        &self.participant
    }

    pub fn shutdown_tasks(&mut self) {
        self.quit.store(true, atomic::Ordering::SeqCst);

        self.sedp_condvar.notify_all();
        self.user_defined_data_send_condvar.notify_all();
        self.announce_sender
            .send(AnnounceKind::DeletedParticipant)
            .ok();

        if let Some(default_unicast_locator) =
            self.participant.default_unicast_locator_list().get(0)
        {
            let port: u32 = default_unicast_locator.port().into();
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port as u16);
            self.sender_socket.send_to(&[0], addr).ok();
        }

        if let Some(metatraffic_unicast_locator) =
            self.participant.metatraffic_unicast_locator_list().get(0)
        {
            let port: u32 = metatraffic_unicast_locator.port().into();
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port as u16);
            self.sender_socket.send_to(&[0], addr).ok();
        }

        if let Some(metatraffic_multicast_transport) =
            self.participant.metatraffic_multicast_locator_list().get(0)
        {
            let addr: [u8; 16] = metatraffic_multicast_transport.address().into();
            let port: u32 = metatraffic_multicast_transport.port().into();
            let addr = SocketAddrV4::new(
                Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                port as u16,
            );
            self.sender_socket.send_to(&[0], addr).ok();
        }

        while let Some(thread) = self.threads.pop() {
            thread.join().unwrap();
        }
    }
}
