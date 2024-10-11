use core::net::{Ipv4Addr, SocketAddr};
use std::{
    collections::HashMap,
    net::UdpSocket,
    sync::{Arc, Mutex},
};

use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use tracing::info;

use crate::{
    domain::domain_participant_factory::DomainId,
    implementation::data_representation_builtin_endpoints::spdp_discovered_participant_data::ParticipantProxy,
    rtps::{
        message_receiver::MessageReceiver, messages::overall_structure::RtpsSubmessageReadKind,
        stateful_writer::RtpsStatefulWriter,
    },
};

use super::{
    discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
    entity::RtpsEntity,
    error::{RtpsError, RtpsErrorKind, RtpsResult},
    message_sender::MessageSender,
    messages::overall_structure::RtpsMessageRead,
    reader::{ReaderHistoryCache, RtpsStatefulReader, RtpsStatelessReader, TransportReader},
    stateful_writer::TransportWriter,
    stateless_writer::RtpsStatelessWriter,
    types::{
        Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_PARTICIPANT,
        LOCATOR_KIND_UDP_V4, PROTOCOLVERSION_2_4, VENDOR_ID_S2E,
    },
};

const MAX_DATAGRAM_SIZE: usize = 65507;

type LocatorAddress = [u8; 16];
// As of 9.6.1.4.1  Default multicast address
const DEFAULT_MULTICAST_LOCATOR_ADDRESS: LocatorAddress =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1];

const PB: i32 = 7400;
const DG: i32 = 250;
#[allow(non_upper_case_globals)]
const d0: i32 = 0;
fn port_builtin_multicast(domain_id: DomainId) -> u16 {
    (PB + DG * domain_id + d0) as u16
}

fn get_multicast_socket(
    multicast_address: LocatorAddress,
    port: u16,
    interface_address_list: impl IntoIterator<Item = Addr>,
) -> std::io::Result<std::net::UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;
    #[cfg(target_family = "unix")]
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;

    socket.bind(&socket_addr.into())?;
    let addr = Ipv4Addr::new(
        multicast_address[12],
        multicast_address[13],
        multicast_address[14],
        multicast_address[15],
    );
    for interface_addr in interface_address_list {
        match interface_addr {
            Addr::V4(a) => {
                let r = socket.join_multicast_v4(&addr, &a.ip);
                if let Err(e) = r {
                    info!(
                        "Failed to join multicast group on address {} with error {}",
                        a.ip, e
                    )
                }
            }
            Addr::V6(_) => (),
        }
    }

    socket.set_multicast_loop_v4(true)?;

    Ok(socket.into())
}

pub fn read_message(
    socket: &mut std::net::UdpSocket,
    buf: &mut [u8],
) -> RtpsResult<RtpsMessageRead> {
    let (bytes, _) = socket.recv_from(buf)?;
    if bytes > 0 {
        Ok(RtpsMessageRead::try_from(&buf[0..bytes])?)
    } else {
        Err(RtpsError::new(RtpsErrorKind::NotEnoughData, ""))
    }
}

pub struct RtpsParticipant {
    entity: RtpsEntity,
    domain_id: DomainId,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    builtin_stateless_writer_list: Arc<Mutex<Vec<Arc<Mutex<RtpsStatelessWriter>>>>>,
    builtin_stateful_writer_list: Arc<Mutex<Vec<Arc<Mutex<RtpsStatefulWriter>>>>>,
    builtin_stateless_reader_list: Arc<Mutex<Vec<Arc<Mutex<RtpsStatelessReader>>>>>,
    user_defined_writer_list: Arc<Mutex<HashMap<[u8; 16], Arc<Mutex<RtpsStatefulWriter>>>>>,
    user_defined_reader_list: Arc<Mutex<HashMap<[u8; 16], Arc<Mutex<RtpsStatefulReader>>>>>,
    sender_socket: UdpSocket,
}

impl RtpsParticipant {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: String,
        interface_name: Option<&String>,
        udp_receive_buffer_size: Option<usize>,
    ) -> RtpsResult<Self> {
        let builtin_stateless_writer_list = Arc::new(Mutex::new(Vec::new()));
        let builtin_stateless_reader_list: Arc<Mutex<Vec<Arc<Mutex<RtpsStatelessReader>>>>> =
            Arc::new(Mutex::new(Vec::new()));
        let builtin_stateful_writer_list =
            Arc::new(Mutex::new(Vec::<Arc<Mutex<RtpsStatefulWriter>>>::new()));
        let user_defined_writer_list = Arc::new(Mutex::new(HashMap::new()));
        let user_defined_reader_list = Arc::new(Mutex::new(HashMap::new()));

        // Open socket for unicast user-defined data
        let interface_address_list = NetworkInterface::show()
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
                    Addr::V4(_) => true,
                    _ => false,
                })
            });

        let default_unicast_socket =
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
        default_unicast_socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())?;
        default_unicast_socket.set_nonblocking(false)?;
        if let Some(buffer_size) = udp_receive_buffer_size {
            default_unicast_socket.set_recv_buffer_size(buffer_size)?;
        }
        let mut default_unicast_socket = std::net::UdpSocket::from(default_unicast_socket);
        let user_defined_unicast_port = default_unicast_socket.local_addr()?.port().into();
        let default_unicast_locator_list = interface_address_list
            .clone()
            .map(|a| Locator::from_ip_and_port(&a, user_defined_unicast_port))
            .collect();

        // Open socket for unicast metatraffic data
        let mut metatrafic_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
        metatrafic_unicast_socket.set_nonblocking(false)?;
        let metattrafic_unicast_locator_port =
            metatrafic_unicast_socket.local_addr()?.port().into();
        let metatraffic_unicast_locator_list: Vec<Locator> = interface_address_list
            .clone()
            .map(|a| Locator::from_ip_and_port(&a, metattrafic_unicast_locator_port))
            .collect();

        // Open socket for multicast metatraffic data
        let metatraffic_multicast_locator_list = vec![Locator::new(
            LOCATOR_KIND_UDP_V4,
            port_builtin_multicast(domain_id) as u32,
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        )];

        let mut metatraffic_multicast_socket = get_multicast_socket(
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            port_builtin_multicast(domain_id),
            interface_address_list,
        )?;
        let builtin_stateless_reader_list_clone = builtin_stateless_reader_list.clone();
        let builtin_stateful_writer_list_clone = builtin_stateful_writer_list.clone();
        std::thread::spawn(move || {
            let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
            loop {
                if let Ok(rtps_message) =
                    read_message(&mut metatraffic_multicast_socket, buf.as_mut_slice())
                {
                    tracing::trace!(
                        rtps_message = ?rtps_message,
                        "Received metatraffic multicast RTPS message"
                    );
                    // let reception_timestamp = self.get_current_time().into();
                    let mut message_receiver = MessageReceiver::new(rtps_message);
                    while let Some(submessage) = message_receiver.next() {
                        match submessage {
                            RtpsSubmessageReadKind::Data(data_submessage) => {
                                for builtin_stateless_reader in
                                    builtin_stateless_reader_list_clone.lock().unwrap().iter()
                                {
                                    builtin_stateless_reader
                                        .lock()
                                        .unwrap()
                                        .on_data_submessage_received(
                                            &data_submessage,
                                            message_receiver.source_guid_prefix(),
                                            message_receiver.source_timestamp(),
                                        );
                                }
                            }
                            //         RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
                            //             let participant_mask_listener = (
                            //                 self.participant_listener_thread
                            //                     .as_ref()
                            //                     .map(|l| l.sender().clone()),
                            //                 self.status_kind.clone(),
                            //             );
                            //             self.builtin_subscriber.send_actor_mail(
                            //                 subscriber_actor::ProcessDataFragSubmessage {
                            //                     data_frag_submessage,
                            //                     source_guid_prefix: message_receiver.source_guid_prefix(),
                            //                     source_timestamp: message_receiver.source_timestamp(),
                            //                     reception_timestamp,
                            //                     subscriber_address: self.builtin_subscriber.address(),
                            //                     participant: message.participant.clone(),
                            //                     participant_mask_listener,
                            //                     executor_handle: message.executor_handle.clone(),
                            //                     timer_handle: self.timer_driver.handle(),
                            //                 },
                            //             );
                            //         }
                            //         RtpsSubmessageReadKind::Gap(gap_submessage) => {
                            //             self.builtin_subscriber.send_actor_mail(
                            //                 subscriber_actor::ProcessGapSubmessage {
                            //                     gap_submessage,
                            //                     source_guid_prefix: message_receiver.source_guid_prefix(),
                            //                 },
                            //             );
                            //         }
                            //         RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => {
                            //             self.builtin_subscriber.send_actor_mail(
                            //                 subscriber_actor::ProcessHeartbeatSubmessage {
                            //                     heartbeat_submessage,
                            //                     source_guid_prefix: message_receiver.source_guid_prefix(),
                            //                     message_sender_actor: self.message_sender_actor.address(),
                            //                 },
                            //             );
                            //         }
                            //         RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => {
                            //             self.builtin_subscriber.send_actor_mail(
                            //                 subscriber_actor::ProcessHeartbeatFragSubmessage {
                            //                     heartbeat_frag_submessage,
                            //                     source_guid_prefix: message_receiver.source_guid_prefix(),
                            //                 },
                            //             );
                            //         }
                            RtpsSubmessageReadKind::AckNack(acknack_submessage) => {
                                for writer in
                                    builtin_stateful_writer_list_clone.lock().unwrap().iter()
                                {
                                    writer.lock().unwrap().on_acknack_submessage_received(
                                        &acknack_submessage,
                                        message_receiver.source_guid_prefix(),
                                    );
                                }
                            }
                            RtpsSubmessageReadKind::NackFrag(nackfrag_submessage) => {
                                for writer in
                                    builtin_stateful_writer_list_clone.lock().unwrap().iter()
                                {
                                    writer.lock().unwrap().on_nack_frag_submessage_received(
                                        &nackfrag_submessage,
                                        message_receiver.source_guid_prefix(),
                                    );
                                }
                            }
                            _ => (),
                        }
                    }

                    // message.executor_handle.spawn(async move {
                    //     process_discovery_data(message.participant.clone())
                    //         .await
                    //         .ok();
                    // });

                    // Ok(())
                    //     let r = participant_address_clone.send_actor_mail(
                    //         domain_participant_actor::ProcessMetatrafficRtpsMessage {
                    //             rtps_message: message,
                    //             participant: participant_clone.clone(),
                    //             executor_handle: participant_clone.executor_handle().clone(),
                    //         },
                    //     );

                    //     if r.is_err() {
                    //         break;
                    //     }
                }
            }
        });

        let builtin_stateful_writer_list_clone = builtin_stateful_writer_list.clone();
        std::thread::spawn(move || {
            let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
            loop {
                if let Ok(rtps_message) =
                    read_message(&mut metatrafic_unicast_socket, buf.as_mut_slice())
                {
                    tracing::trace!(
                        rtps_message = ?rtps_message,
                        "Received metatraffic unicast RTPS message"
                    );

                    let mut message_receiver = MessageReceiver::new(rtps_message);
                    //             let r = participant_address_clone.send_actor_mail(
                    //                 domain_participant_actor::ProcessMetatrafficRtpsMessage {
                    //                     rtps_message: message,
                    //                     participant: participant_clone.clone(),
                    //                     executor_handle: participant_clone.executor_handle().clone(),
                    //                 },
                    //             );

                    //             if r.is_err() {
                    //                 break;
                    //             }
                }
            }
        });

        let user_defined_writer_list_clone = user_defined_writer_list.clone();
        std::thread::spawn(move || {
            let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
            loop {
                if let Ok(rtps_message) =
                    read_message(&mut default_unicast_socket, buf.as_mut_slice())
                {
                    tracing::trace!(
                        rtps_message = ?rtps_message,
                        "Received user defined data unicast RTPS message"
                    );
                    //             let r = participant_address_clone.send_actor_mail(
                    //                 domain_participant_actor::ProcessUserDefinedRtpsMessage {
                    //                     rtps_message: message,
                    //                     participant: participant_clone.clone(),
                    //                     executor_handle: participant_clone.executor_handle().clone(),
                    //                 },
                    //             );
                    //             if r.is_err() {
                    //                 break;
                    //             }
                }
            }
        });

        let sender_socket = std::net::UdpSocket::bind("0.0.0.0:0000")?;
        Ok(Self {
            entity: RtpsEntity::new(Guid::new(guid_prefix, ENTITYID_PARTICIPANT)),
            domain_id,
            domain_tag,
            protocol_version: PROTOCOLVERSION_2_4,
            vendor_id: VENDOR_ID_S2E,
            default_unicast_locator_list,
            default_multicast_locator_list: Vec::new(),
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            builtin_stateless_writer_list,
            builtin_stateful_writer_list,
            builtin_stateless_reader_list,
            user_defined_writer_list,
            user_defined_reader_list,
            sender_socket,
        })
    }

    pub fn guid(&self) -> Guid {
        self.entity.guid()
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.default_unicast_locator_list.as_slice()
    }

    pub fn set_default_unicast_locator_list(&mut self, list: Vec<Locator>) {
        self.default_unicast_locator_list = list;
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.default_multicast_locator_list.as_slice()
    }

    pub fn set_default_multicast_locator_list(&mut self, list: Vec<Locator>) {
        self.default_multicast_locator_list = list;
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_unicast_locator_list.as_ref()
    }

    pub fn set_metatraffic_unicast_locator_list(&mut self, list: Vec<Locator>) {
        self.metatraffic_unicast_locator_list = list;
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_multicast_locator_list.as_ref()
    }

    pub fn set_metatraffic_multicast_locator_list(&mut self, list: Vec<Locator>) {
        self.metatraffic_multicast_locator_list = list;
    }

    pub fn create_builtin_stateless_writer(
        &mut self,
        writer_guid: Guid,
    ) -> Arc<Mutex<RtpsStatelessWriter>> {
        let socket = self
            .sender_socket
            .try_clone()
            .expect("Should always be clone");
        let message_sender = MessageSender::new(self.entity.guid().prefix(), socket);
        let mut stateless_writer = RtpsStatelessWriter::new(writer_guid, message_sender);

        let spdp_discovery_locator_list = [Locator::new(
            LOCATOR_KIND_UDP_V4,
            port_builtin_multicast(self.domain_id) as u32,
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        )];

        for reader_locator in spdp_discovery_locator_list {
            stateless_writer.reader_locator_add(reader_locator);
        }

        let writer = Arc::new(Mutex::new(stateless_writer));
        self.builtin_stateless_writer_list
            .lock()
            .unwrap()
            .push(writer.clone());
        writer
    }

    pub fn create_builtin_stateful_writer(
        &mut self,
        writer_guid: Guid,
    ) -> Arc<Mutex<RtpsStatefulWriter>> {
        let socket = self
            .sender_socket
            .try_clone()
            .expect("Should always be clone");
        let message_sender = MessageSender::new(self.entity.guid().prefix(), socket);
        let writer = Arc::new(Mutex::new(RtpsStatefulWriter::new(
            writer_guid,
            message_sender,
        )));
        self.builtin_stateful_writer_list
            .lock()
            .unwrap()
            .push(writer.clone());
        writer
    }

    pub fn create_builtin_stateless_reader(
        &mut self,
        reader_guid: Guid,
        history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<RtpsStatelessReader>> {
        let stateless_reader = RtpsStatelessReader::new(reader_guid, history_cache);

        let reader = Arc::new(Mutex::new(stateless_reader));
        self.builtin_stateless_reader_list
            .lock()
            .unwrap()
            .push(reader.clone());
        reader
    }
}

impl RtpsParticipant {
    pub fn participant_proxy(&self) -> ParticipantProxy {
        ParticipantProxy {
            domain_id: Some(self.domain_id),
            domain_tag: self.domain_tag.clone(),
            protocol_version: PROTOCOLVERSION_2_4,
            guid_prefix: self.entity.guid().prefix(),
            vendor_id: VENDOR_ID_S2E,
            expects_inline_qos: false,
            metatraffic_unicast_locator_list: self.metatraffic_unicast_locator_list.clone(),
            metatraffic_multicast_locator_list: self.metatraffic_multicast_locator_list.clone(),
            default_unicast_locator_list: self.default_unicast_locator_list.clone(),
            default_multicast_locator_list: self.default_multicast_locator_list.clone(),
            available_builtin_endpoints: BuiltinEndpointSet::default(),
            manual_liveliness_count: 0,
            builtin_endpoint_qos: BuiltinEndpointQos::default(),
        }
    }

    pub fn create_writer(
        &mut self,
        writer_guid: Guid,
    ) -> Arc<Mutex<dyn TransportWriter + Send + Sync + 'static>> {
        let socket = self
            .sender_socket
            .try_clone()
            .expect("Should always be clone");
        let message_sender = MessageSender::new(self.entity.guid().prefix(), socket);
        let writer: Arc<Mutex<RtpsStatefulWriter>> = Arc::new(Mutex::new(RtpsStatefulWriter::new(
            writer_guid,
            message_sender,
        )));
        self.user_defined_writer_list
            .lock()
            .unwrap()
            .insert(writer_guid.into(), writer.clone());
        writer
    }

    pub fn delete_writer(&mut self, writer_guid: Guid) {
        self.user_defined_writer_list
            .lock()
            .unwrap()
            .remove(&<[u8; 16]>::from(writer_guid));
    }

    pub fn create_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader + Send + Sync + 'static>> {
        todo!()
    }

    pub fn delete_reader(&mut self, writer_guid: Guid) {
        self.user_defined_reader_list
            .lock()
            .unwrap()
            .remove(&<[u8; 16]>::from(writer_guid));
    }
}
