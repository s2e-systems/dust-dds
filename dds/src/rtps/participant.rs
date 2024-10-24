use core::net::{Ipv4Addr, SocketAddr};
use std::{
    net::UdpSocket,
    sync::{Arc, Mutex},
};

use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use tracing::info;

use crate::{
    domain::domain_participant_factory::DomainId,
    implementation::data_representation_builtin_endpoints::{
        discovered_reader_data::ReaderProxy,
        discovered_writer_data::WriterProxy,
        spdp_discovered_participant_data::{ParticipantProxy, SpdpDiscoveredParticipantData},
    },
    rtps::{
        discovery_types::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        },
        message_receiver::MessageReceiver,
        stateful_writer::RtpsStatefulWriter,
        types::ENTITYID_UNKNOWN,
    },
};

use super::{
    behavior_types::InstanceHandle,
    discovery_types::{
        BuiltinEndpointQos, BuiltinEndpointSet, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
    },
    entity::RtpsEntity,
    error::{RtpsError, RtpsErrorKind, RtpsResult},
    message_sender::MessageSender,
    messages::overall_structure::RtpsMessageRead,
    reader::{ReaderHistoryCache, RtpsStatefulReader, RtpsStatelessReader, TransportReader},
    stateful_writer::TransportWriter,
    stateless_writer::RtpsStatelessWriter,
    types::{
        EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, TopicKind, VendorId,
        ENTITYID_PARTICIPANT, LOCATOR_KIND_UDP_V4, PROTOCOLVERSION_2_4, USER_DEFINED_READER_NO_KEY,
        USER_DEFINED_READER_WITH_KEY, USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY,
        VENDOR_ID_S2E,
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
    builtin_stateful_reader_list: Arc<Mutex<Vec<Arc<Mutex<RtpsStatefulReader>>>>>,
    user_defined_writer_list: Arc<Mutex<Vec<Arc<Mutex<RtpsStatefulWriter>>>>>,
    user_defined_reader_list: Arc<Mutex<Vec<Arc<Mutex<RtpsStatefulReader>>>>>,
    sender_socket: UdpSocket,
    endpoint_counter: u8,
    discovered_participant_list: Vec<InstanceHandle>,
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
        let builtin_stateless_reader_list =
            Arc::new(Mutex::new(Vec::<Arc<Mutex<RtpsStatelessReader>>>::new()));
        let builtin_stateful_writer_list =
            Arc::new(Mutex::new(Vec::<Arc<Mutex<RtpsStatefulWriter>>>::new()));
        let builtin_stateful_reader_list =
            Arc::new(Mutex::new(Vec::<Arc<Mutex<RtpsStatefulReader>>>::new()));
        let user_defined_writer_list =
            Arc::new(Mutex::new(Vec::<Arc<Mutex<RtpsStatefulWriter>>>::new()));
        let user_defined_reader_list =
            Arc::new(Mutex::new(Vec::<Arc<Mutex<RtpsStatefulReader>>>::new()));

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
        let mut metatraffic_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
        metatraffic_unicast_socket.set_nonblocking(false)?;
        let metattrafic_unicast_locator_port =
            metatraffic_unicast_socket.local_addr()?.port().into();
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
        let builtin_stateful_reader_list_clone = builtin_stateful_reader_list.clone();
        let builtin_stateful_writer_list_clone = builtin_stateful_writer_list.clone();
        std::thread::Builder::new()
            .name("RTPS metatraffic multicast discovery".to_string())
            .spawn(move || {
                let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut metatraffic_multicast_socket, buf.as_mut_slice())
                    {
                        tracing::trace!(
                            rtps_message = ?rtps_message,
                            "Received metatraffic multicast RTPS message"
                        );
                        MessageReceiver::new(rtps_message).process_message(
                            builtin_stateless_reader_list_clone.lock().unwrap().as_ref(),
                            builtin_stateful_reader_list_clone.lock().unwrap().as_ref(),
                            builtin_stateful_writer_list_clone.lock().unwrap().as_ref(),
                        );
                    }
                }
            })
            .expect("failed to spawn thread");

        let builtin_stateless_reader_list_clone = builtin_stateless_reader_list.clone();
        let builtin_stateful_reader_list_clone = builtin_stateful_reader_list.clone();
        let builtin_stateful_writer_list_clone = builtin_stateful_writer_list.clone();
        std::thread::Builder::new()
            .name("RTPS metatraffic unicast discovery".to_string())
            .spawn(move || {
                let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut metatraffic_unicast_socket, buf.as_mut_slice())
                    {
                        tracing::trace!(
                            rtps_message = ?rtps_message,
                            "Received metatraffic unicast RTPS message"
                        );

                        MessageReceiver::new(rtps_message).process_message(
                            builtin_stateless_reader_list_clone.lock().unwrap().as_ref(),
                            builtin_stateful_reader_list_clone.lock().unwrap().as_ref(),
                            builtin_stateful_writer_list_clone.lock().unwrap().as_ref(),
                        );
                    }
                }
            })
            .expect("failed to spawn thread");

        let user_defined_reader_list_clone = user_defined_reader_list.clone();
        let user_defined_writer_list_clone = user_defined_writer_list.clone();
        std::thread::Builder::new()
            .name("RTPS user defined traffic".to_string())
            .spawn(move || {
                let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut default_unicast_socket, buf.as_mut_slice())
                    {
                        tracing::trace!(
                            rtps_message = ?rtps_message,
                            "Received user defined data unicast RTPS message"
                        );
                        MessageReceiver::new(rtps_message).process_message(
                            &[],
                            user_defined_reader_list_clone.lock().unwrap().as_ref(),
                            user_defined_writer_list_clone.lock().unwrap().as_ref(),
                        );
                    }
                }
            })
            .expect("failed to spawn thread");

        // Heartbeat thread
        let builtin_stateful_writer_list_clone = builtin_stateful_writer_list.clone();
        let user_defined_writer_list_clone = user_defined_writer_list.clone();
        std::thread::Builder::new()
            .name("RTPS user defined heartbeat".to_string())
            .spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(50));
                for builtin_writer in builtin_stateful_writer_list_clone.lock().unwrap().iter() {
                    builtin_writer.lock().unwrap().send_message();
                }
                for user_defined_writer in user_defined_writer_list_clone.lock().unwrap().iter() {
                    user_defined_writer.lock().unwrap().send_message();
                }
            })
            .expect("failed to spawn thread");

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
            builtin_stateful_reader_list,
            user_defined_writer_list,
            user_defined_reader_list,
            sender_socket,
            endpoint_counter: 0,
            discovered_participant_list: Vec::new(),
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
        let mut stateless_writer =
            RtpsStatelessWriter::new(writer_guid, message_sender, self.participant_proxy());

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
        let reader = Arc::new(Mutex::new(RtpsStatelessReader::new(
            reader_guid,
            history_cache,
        )));
        self.builtin_stateless_reader_list
            .lock()
            .unwrap()
            .push(reader.clone());
        reader
    }

    pub fn create_builtin_stateful_reader(
        &mut self,
        reader_guid: Guid,
        history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<RtpsStatefulReader>> {
        let socket = self
            .sender_socket
            .try_clone()
            .expect("Should always be clone");
        let message_sender = MessageSender::new(self.entity.guid().prefix(), socket);
        let reader = Arc::new(Mutex::new(RtpsStatefulReader::new(
            reader_guid,
            history_cache,
            message_sender,
        )));
        self.builtin_stateful_reader_list
            .lock()
            .unwrap()
            .push(reader.clone());
        reader
    }

    pub fn add_discovered_participant(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // IN CASE no domain id was transmitted the a local domain id is assumed
        // (as specified in Table 9.19 - ParameterId mapping and default values)
        let is_domain_id_matching = discovered_participant_data
            .participant_proxy
            .domain_id
            .unwrap_or(self.domain_id)
            == self.domain_id;
        let is_domain_tag_matching =
            discovered_participant_data.participant_proxy.domain_tag == self.domain_tag;
        let discovered_participant_handle =
            InstanceHandle(discovered_participant_data.dds_participant_data.key().value);

        let is_participant_discovered = self
            .discovered_participant_list
            .contains(&discovered_participant_handle);
        if is_domain_id_matching && is_domain_tag_matching && !is_participant_discovered {
            self.add_matched_publications_detector(&discovered_participant_data);
            self.add_matched_publications_announcer(&discovered_participant_data);
            self.add_matched_subscriptions_detector(&discovered_participant_data);
            self.add_matched_subscriptions_announcer(&discovered_participant_data);
            self.add_matched_topics_detector(&discovered_participant_data);
            self.add_matched_topics_announcer(&discovered_participant_data);
        }
    }

    fn add_matched_publications_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        // participant: DomainParticipantAsync,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            todo!()
        }
    }

    fn add_matched_publications_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            todo!()
            // self.builtin_subscriber
            //     .add_matched_writer(&discovered_writer_data);
        }
    }

    fn add_matched_subscriptions_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };

            // self.builtin_publisher
            //     .add_matched_reader(&discovered_reader_data);
            todo!()
        }
    }

    fn add_matched_subscriptions_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        // participant: DomainParticipantAsync,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            todo!()
            // self.builtin_subscriber
            //     .add_matched_writer(&discovered_writer_data);
        }
    }

    fn add_matched_topics_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            todo!()
            // self.builtin_publisher
            //     .add_matched_reader(&discovered_reader_data);
        }
    }

    fn add_matched_topics_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            todo!()

            // self.builtin_subscriber
            //     .add_matched_writer(&discovered_writer_data);
        }
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
        topic_kind: TopicKind,
    ) -> Arc<Mutex<dyn TransportWriter + Send + Sync + 'static>> {
        let entity_kind = match topic_kind {
            TopicKind::WithKey => USER_DEFINED_WRITER_WITH_KEY,
            TopicKind::NoKey => USER_DEFINED_WRITER_NO_KEY,
        };
        self.endpoint_counter += 1;
        let data_writer_counter = self.endpoint_counter;
        let entity_key: [u8; 3] = [0, 0, data_writer_counter];
        let entity_id = EntityId::new(entity_key, entity_kind);
        let writer_guid = Guid::new(self.guid().prefix(), entity_id);

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
            .push(writer.clone());
        writer
    }

    pub fn delete_writer(&mut self, writer_guid: Guid) {
        self.user_defined_writer_list
            .lock()
            .unwrap()
            .retain(|x| x.lock().unwrap().guid() != writer_guid);
    }

    pub fn create_reader(
        &mut self,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader + Send + Sync + 'static>> {
        // let subscriber_guid = self.rtps_group.guid();
        let entity_kind = match topic_kind {
            TopicKind::WithKey => USER_DEFINED_READER_WITH_KEY,
            TopicKind::NoKey => USER_DEFINED_READER_NO_KEY,
        };
        self.endpoint_counter += 1;
        let data_reader_counter = self.endpoint_counter;
        let entity_key: [u8; 3] = [0, data_reader_counter, 0];
        let entity_id = EntityId::new(entity_key, entity_kind);
        let reader_guid = Guid::new(self.guid().prefix(), entity_id);
        let socket = self
            .sender_socket
            .try_clone()
            .expect("Should always be clone");
        let message_sender = MessageSender::new(self.entity.guid().prefix(), socket);
        let reader = Arc::new(Mutex::new(RtpsStatefulReader::new(
            reader_guid,
            reader_history_cache,
            message_sender,
        )));
        self.user_defined_reader_list
            .lock()
            .unwrap()
            .push(reader.clone());
        reader
    }

    pub fn delete_reader(&mut self, reader_guid: Guid) {
        self.user_defined_reader_list
            .lock()
            .unwrap()
            .retain(|x| x.lock().unwrap().guid() != reader_guid);
    }
}

// let spdp_builtin_participant_writer = todo!();
// let spdp_builtin_participant_reader = RtpsStatelessReader::new(
//     Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
//     spdp_builtin_participant_reader_history_cache,
// );
// let sedp_builtin_topics_writer = todo!();
// let message_sender = todo!();
// let sedp_builtin_topics_reader = RtpsStatefulReader::new(
//     Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
//     sedp_builtin_topics_reader_history_cache,
//     message_sender,
// );
// let sedp_builtin_publications_writer = todo!();
// let message_sender = todo!();
// let sedp_builtin_publications_reader = RtpsStatefulReader::new(
//     Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
//     sedp_builtin_publications_reader_history_cache,
//     message_sender,
// );
// let sedp_builtin_subscriptions_writer = todo!();
// let sedp_builtin_subscriptions_reader = RtpsStatefulReader::new(
//     Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
//     sedp_builtin_subscriptions_reader_history_cache,
//     message_sender,
// );
