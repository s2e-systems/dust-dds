use std::{
    net::UdpSocket,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread::JoinHandle,
};

use crate::{
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        rtps::participant::RtpsParticipant,
        rtps_udp_psm::udp_transport::{RtpsUdpPsm, UdpTransport},
        utils::{condvar::DdsCondvar, shared_object::DdsShared},
    },
    infrastructure::{
        error::DdsResult, qos::DomainParticipantQos, status::StatusKind, time::Duration,
    },
};

use super::{configuration::DustDdsConfiguration, domain_participant_impl::DomainParticipantImpl};

pub struct DcpsService {
    participant: DdsShared<DomainParticipantImpl>,
    quit: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl DcpsService {
    pub fn new(
        rtps_participant: RtpsParticipant,
        domain_id: DomainId,
        configuration: DustDdsConfiguration,
        domain_participant_qos: DomainParticipantQos,
        listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
        mut transport: RtpsUdpPsm,
    ) -> DdsResult<Self> {
        let announcer_condvar = DdsCondvar::new();
        let sedp_condvar = DdsCondvar::new();
        let user_defined_data_send_condvar = DdsCondvar::new();
        let participant = DomainParticipantImpl::new(
            rtps_participant,
            domain_id,
            configuration.domain_tag,
            domain_participant_qos,
            listener,
            mask,
            transport.metatraffic_unicast_locator_list(),
            vec![],
            transport.metatraffic_multicast_locator_list().as_slice(),
            announcer_condvar.clone(),
            sedp_condvar.clone(),
            user_defined_data_send_condvar.clone(),
        );

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
            let mut metatraffic_multicast_transport =
                transport.metatraffic_multicast_transport().unwrap();
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
            let mut metatraffic_unicast_transport =
                transport.metatraffic_unicast_transport().unwrap();
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
            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }
                let _r = sedp_condvar.wait_timeout(Duration::new(0, 500000000));

                domain_participant.send_built_in_data(&mut metatraffic_unicast_transport_send);
            }));
        }

        // //////////// User-defined Communication receive
        {
            let domain_participant = participant.clone();
            let mut default_unicast_transport = transport.default_unicast_transport().unwrap();
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

            threads.push(std::thread::spawn(move || loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                let _r = user_defined_data_send_condvar.wait_timeout(Duration::new(0, 100_000_000));

                domain_participant.send_user_defined_data(&mut default_unicast_transport_send);
            }));
        }

        // //////////// Announce participant
        let domain_participant = participant.clone();
        let task_quit = quit.clone();

        threads.push(std::thread::spawn(move || {
            // TODO: Temporary solution to ensure tests pass by announcing as soon as possible
            domain_participant.announce_participant().ok();
            loop {
                if task_quit.load(atomic::Ordering::SeqCst) {
                    break;
                }

                let _r = announcer_condvar.wait_timeout(Duration::new(5, 0));

                match domain_participant.announce_participant() {
                    Ok(_) => (),
                    Err(e) => println!("participant announcement failed: {:?}", e),
                }
            }
        }));

        Ok(DcpsService {
            participant,
            quit,
            threads,
        })
    }

    pub fn participant(&self) -> &DdsShared<DomainParticipantImpl> {
        &self.participant
    }

    pub fn shutdown_tasks(&mut self) {
        self.quit.store(true, atomic::Ordering::SeqCst);

        while let Some(thread) = self.threads.pop() {
            thread.join().unwrap();
        }
    }
}
