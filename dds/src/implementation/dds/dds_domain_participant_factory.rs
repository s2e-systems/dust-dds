use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    str::FromStr,
    sync::atomic,
};

use jsonschema::JSONSchema;
use lazy_static::lazy_static;
use mac_address::MacAddress;
use schemars::schema_for;
use socket2::Socket;

use crate::{
    domain::{
        domain_participant::{
            task_metatraffic_multicast_receive, task_metatraffic_unicast_receive,
            task_send_entity_announce, task_unicast_metatraffic_communication_send,
            task_unicast_user_defined_communication_send, task_user_defined_receive,
            DomainParticipant,
        },
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        configuration::DustDdsConfiguration,
        rtps::{
            participant::RtpsParticipant,
            types::{
                GuidPrefix, Locator, LocatorAddress, LocatorPort, LOCATOR_KIND_UDP_V4,
                PROTOCOLVERSION, VENDOR_ID_S2E,
            },
        },
        rtps_udp_psm::udp_transport::UdpTransportRead,
        utils::{
            actor::{spawn_actor, Actor, ActorAddress, ActorJoinHandle, Handler, Message},
            condvar::DdsCondvar,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};

use super::{
    dds_domain_participant::{self, DdsDomainParticipant},
    nodes::DomainParticipantNode,
};

lazy_static! {
    pub static ref THE_DDS_CONFIGURATION: DustDdsConfiguration =
        if let Ok(configuration_json) = std::env::var("DUST_DDS_CONFIGURATION") {
            configuration_try_from_str(configuration_json.as_str()).unwrap()
        } else {
            DustDdsConfiguration::default()
        };
}

pub struct DdsDomainParticipantFactory {
    domain_participant_list: Vec<(ActorAddress<DdsDomainParticipant>, ActorJoinHandle)>,
    domain_participant_counter: u32,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
}

impl Default for DdsDomainParticipantFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for DdsDomainParticipantFactory {}

pub struct CreateParticipant {
    domain_id: DomainId,
    qos: QosKind<DomainParticipantQos>,
    a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
    mask: Vec<StatusKind>,
}

impl CreateParticipant {
    pub fn new(
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            domain_id,
            qos,
            a_listener,
            mask,
        }
    }
}

impl Message for CreateParticipant {
    type Result = ActorAddress<DdsDomainParticipant>;
}

impl Handler<CreateParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: CreateParticipant) -> <CreateParticipant as Message>::Result {
        let domain_participant_qos = match message.qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let mac_address = ifcfg::IfCfg::get()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter_map(|i| MacAddress::from_str(&i.mac).ok())
            .find(|&mac| mac != MacAddress::new([0, 0, 0, 0, 0, 0]))
            .expect("Could not find any mac address")
            .bytes();

        let app_id = std::process::id().to_ne_bytes();
        let instance_id = self.domain_participant_counter.to_ne_bytes();
        self.domain_participant_counter += 1;

        #[rustfmt::skip]
        let guid_prefix = GuidPrefix::new([
            mac_address[2],  mac_address[3], mac_address[4], mac_address[5], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
        ]);

        let default_unicast_locator_list = vec![];
        let default_multicast_locator_list = vec![];
        let metatraffic_unicast_locator_list = vec![];
        let metatraffic_multicast_locator_list = vec![];
        let spdp_discovery_locator_list = vec![];
        let sedp_condvar = DdsCondvar::new();
        let user_defined_data_send_condvar = DdsCondvar::new();
        let (announce_sender, _announce_receiver) = tokio::sync::mpsc::channel(500);

        let rtps_participant = RtpsParticipant::new(
            guid_prefix,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
        );

        let domain_participant = DdsDomainParticipant::new(
            rtps_participant,
            message.domain_id,
            THE_DDS_CONFIGURATION.domain_tag.clone(),
            domain_participant_qos,
            &spdp_discovery_locator_list,
            user_defined_data_send_condvar.clone(),
            THE_DDS_CONFIGURATION.fragment_size,
            announce_sender,
            sedp_condvar.clone(),
        );

        let (participant_address, participant_join_handle) = spawn_actor(domain_participant);

        self.domain_participant_list
            .push((participant_address.clone(), participant_join_handle));

        if self.qos.entity_factory.autoenable_created_entities {
            todo!()
            // participant_address
            //     .send(dds_domain_participant::Enable)
            //     .await;
        }

        participant_address
    }
    // let interface_address_list =
    //     get_interface_address_list(configuration.interface_name.as_ref());

    // let default_unicast_socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
    //     .map_err(|_| DdsError::Error)?;
    // default_unicast_socket.set_nonblocking(true).unwrap();
    // let user_defined_unicast_port = default_unicast_socket
    //     .local_addr()
    //     .map_err(|_| DdsError::Error)?
    //     .port();
    // let user_defined_unicast_locator_port = LocatorPort::new(user_defined_unicast_port.into());

    // let default_unicast_locator_list: Vec<Locator> = interface_address_list
    //     .iter()
    //     .map(|a| Locator::new(LOCATOR_KIND_UDP_V4, user_defined_unicast_locator_port, *a))
    //     .collect();

    // let default_multicast_locator_list = vec![];

    // let metattrafic_unicast_socket =
    //     UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
    //         .map_err(|_| DdsError::Error)?;
    // metattrafic_unicast_socket.set_nonblocking(true).unwrap();

    // let metattrafic_unicast_locator_port = LocatorPort::new(
    //     metattrafic_unicast_socket
    //         .local_addr()
    //         .map_err(|_| DdsError::Error)?
    //         .port()
    //         .into(),
    // );
    // let metatraffic_unicast_locator_list: Vec<Locator> = interface_address_list
    //     .iter()
    //     .map(|a| Locator::new(LOCATOR_KIND_UDP_V4, metattrafic_unicast_locator_port, *a))
    //     .collect();

    // let metatraffic_multicast_locator_list = vec![Locator::new(
    //     LOCATOR_KIND_UDP_V4,
    //     port_builtin_multicast(domain_id),
    //     DEFAULT_MULTICAST_LOCATOR_ADDRESS,
    // )];

    // let spdp_discovery_locator_list = metatraffic_multicast_locator_list.clone();

    // let mac_address = ifcfg::IfCfg::get()
    //     .expect("Could not scan interfaces")
    //     .into_iter()
    //     .filter_map(|i| MacAddress::from_str(&i.mac).ok())
    //     .find(|&mac| mac != MacAddress::new([0, 0, 0, 0, 0, 0]))
    //     .expect("Could not find any mac address")
    //     .bytes();

    // let app_id = std::process::id().to_ne_bytes();
    // let instance_id = self
    //     .domain_participant_counter
    //     .fetch_add(1, atomic::Ordering::SeqCst)
    //     .to_ne_bytes();

    // #[rustfmt::skip]
    // let guid_prefix = GuidPrefix::new([
    //     mac_address[2],  mac_address[3], mac_address[4], mac_address[5], // Host ID
    //     app_id[0], app_id[1], app_id[2], app_id[3], // App ID
    //     instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
    // ]);

    // let rtps_participant = RtpsParticipant::new(
    //     guid_prefix,
    //     default_unicast_locator_list,
    //     default_multicast_locator_list,
    //     metatraffic_unicast_locator_list,
    //     metatraffic_multicast_locator_list,
    //     PROTOCOLVERSION,
    //     VENDOR_ID_S2E,
    // );
    // let guid = rtps_participant.guid();
    // let sedp_condvar = DdsCondvar::new();
    // let user_defined_data_send_condvar = DdsCondvar::new();
    // let (announce_sender, announce_receiver) = tokio::sync::mpsc::channel(500);

    // let mut dds_participant = DdsDomainParticipant::new(
    //     rtps_participant,
    //     domain_id,
    //     configuration.domain_tag,
    //     domain_participant_qos,
    //     &spdp_discovery_locator_list,
    //     user_defined_data_send_condvar.clone(),
    //     configuration.fragment_size,
    //     announce_sender,
    //     sedp_condvar.clone(),
    // );
    // let _guard = THE_TASK_RUNTIME.enter();
    // dds_participant.spawn(task_send_entity_announce(guid_prefix, announce_receiver));

    // self.add_domain_participant_listener(guid, a_listener, mask);

    // let (listener_sender, listener_receiver) = tokio::sync::mpsc::channel(500);

    // dds_participant.spawn(
    //     crate::domain::domain_participant::task_update_communication_status(
    //         guid_prefix,
    //         listener_sender.clone(),
    //     ),
    // );

    // dds_participant.spawn(crate::domain::domain_participant::task_listener_receiver(
    //     listener_receiver,
    // ));

    // let metatraffic_multicast_transport = UdpTransportRead::new(
    //     get_multicast_socket(
    //         DEFAULT_MULTICAST_LOCATOR_ADDRESS,
    //         port_builtin_multicast(domain_id),
    //     )
    //     .unwrap(),
    // );
    // dds_participant.spawn(task_metatraffic_multicast_receive(
    //     guid_prefix,
    //     metatraffic_multicast_transport,
    //     dds_participant.sedp_condvar().clone(),
    //     listener_sender.clone(),
    // ));

    // let metatraffic_unicast_transport = UdpTransportRead::new(
    //     tokio::net::UdpSocket::from_std(metattrafic_unicast_socket).unwrap(),
    // );

    // dds_participant.spawn(task_metatraffic_unicast_receive(
    //     guid_prefix,
    //     metatraffic_unicast_transport,
    //     dds_participant.sedp_condvar().clone(),
    //     listener_sender.clone(),
    // ));

    // let default_unicast_transport =
    //     UdpTransportRead::new(tokio::net::UdpSocket::from_std(default_unicast_socket).unwrap());

    // dds_participant.spawn(task_user_defined_receive(
    //     guid_prefix,
    //     default_unicast_transport,
    //     listener_sender,
    // ));

    // dds_participant.spawn(task_unicast_metatraffic_communication_send(
    //     guid_prefix,
    //     sedp_condvar,
    // ));

    // dds_participant.spawn(task_unicast_user_defined_communication_send(
    //     guid_prefix,
    //     user_defined_data_send_condvar,
    // ));

    // self.domain_participant_list
    //     .write()
    //     .await
    //     .insert(guid_prefix, dds_participant);

    // Ok(DomainParticipant::new(DomainParticipantNode::new(guid)))
}

pub struct DeleteParticipant;

impl Message for DeleteParticipant {
    type Result = ();
}

impl Handler<DeleteParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: DeleteParticipant) -> <DeleteParticipant as Message>::Result {
        todo!()
    }
}

pub struct LookupParticipant {
    domain_id: DomainId,
}

impl LookupParticipant {
    pub fn new(domain_id: DomainId) -> Self {
        Self { domain_id }
    }
}

impl Message for LookupParticipant {
    type Result = Option<ActorAddress<DdsDomainParticipant>>;
}

impl Handler<LookupParticipant> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: LookupParticipant) -> <LookupParticipant as Message>::Result {
        todo!()
    }
}

pub struct GetQos;

impl Message for GetQos {
    type Result = DomainParticipantFactoryQos;
}

impl Handler<GetQos> for DdsDomainParticipantFactory {
    fn handle(&mut self, _message: GetQos) -> <GetQos as Message>::Result {
        self.qos.clone()
    }
}

pub struct SetQos {
    qos_kind: QosKind<DomainParticipantFactoryQos>,
}

impl SetQos {
    pub fn new(qos_kind: QosKind<DomainParticipantFactoryQos>) -> Self {
        Self { qos_kind }
    }
}

impl Message for SetQos {
    type Result = ();
}

impl Handler<SetQos> for DdsDomainParticipantFactory {
    fn handle(&mut self, message: SetQos) -> <SetQos as Message>::Result {
        let qos = match message.qos_kind {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
    }
}

pub struct GetDefaultParticipantQos;

impl Message for GetDefaultParticipantQos {
    type Result = DomainParticipantQos;
}

impl Handler<GetDefaultParticipantQos> for DdsDomainParticipantFactory {
    fn handle(
        &mut self,
        _message: GetDefaultParticipantQos,
    ) -> <GetDefaultParticipantQos as Message>::Result {
        self.default_participant_qos.clone()
    }
}

pub struct SetDefaultParticipantQos {
    qos_kind: QosKind<DomainParticipantQos>,
}

impl SetDefaultParticipantQos {
    pub fn new(qos_kind: QosKind<DomainParticipantQos>) -> Self {
        Self { qos_kind }
    }
}

impl Message for SetDefaultParticipantQos {
    type Result = ();
}

impl Handler<SetDefaultParticipantQos> for DdsDomainParticipantFactory {
    fn handle(
        &mut self,
        message: SetDefaultParticipantQos,
    ) -> <SetDefaultParticipantQos as Message>::Result {
        let qos = match message.qos_kind {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };
        self.default_participant_qos = qos;
    }
}

impl DdsDomainParticipantFactory {
    pub fn new() -> Self {
        Self {
            domain_participant_list: Vec::new(),
            domain_participant_counter: 0,
            qos: DomainParticipantFactoryQos::default(),
            default_participant_qos: DomainParticipantQos::default(),
        }
    }
}

// As of 9.6.1.4.1  Default multicast address
const DEFAULT_MULTICAST_LOCATOR_ADDRESS: LocatorAddress =
    LocatorAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1]);

const PB: i32 = 7400;
const DG: i32 = 250;
#[allow(non_upper_case_globals)]
const d0: i32 = 0;

fn port_builtin_multicast(domain_id: DomainId) -> LocatorPort {
    LocatorPort::new((PB + DG * domain_id + d0) as u32)
}

fn get_interface_address_list(interface_name: Option<&String>) -> Vec<LocatorAddress> {
    ifcfg::IfCfg::get()
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
            i.addresses.into_iter().filter_map(|a| match a.address? {
                #[rustfmt::skip]
                SocketAddr::V4(v4) if !v4.ip().is_loopback() => Some(
                    LocatorAddress::new([0, 0, 0, 0,
                        0, 0, 0, 0,
                        0, 0, 0, 0,
                        v4.ip().octets()[0], v4.ip().octets()[1], v4.ip().octets()[2], v4.ip().octets()[3]])
                    ),
                _ => None,
            })
        })
        .collect()
}

fn get_multicast_socket(
    multicast_address: LocatorAddress,
    port: LocatorPort,
) -> std::io::Result<tokio::net::UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, <u32>::from(port) as u16));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
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

    tokio::net::UdpSocket::from_std(socket.into())
}

fn configuration_try_from_str(configuration_json: &str) -> Result<DustDdsConfiguration, String> {
    let root_schema = schema_for!(DustDdsConfiguration);
    let json_schema_str =
        serde_json::to_string(&root_schema).expect("Json schema could not be created");

    let schema = serde_json::value::Value::from_str(json_schema_str.as_str())
        .expect("Json schema not valid");
    let compiled_schema = JSONSchema::compile(&schema).expect("Json schema could not be compiled");

    let instance =
        serde_json::value::Value::from_str(configuration_json).map_err(|e| e.to_string())?;
    compiled_schema
        .validate(&instance)
        .map_err(|errors| errors.map(|e| e.to_string()).collect::<String>())?;
    serde_json::from_value(instance).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_configuration_json() {
        let configuration = configuration_try_from_str(
            r#"{"domain_tag" : "from_configuration_json", "interface_name": "Wi-Fi"}"#,
        )
        .unwrap();
        assert_eq!(
            configuration,
            DustDdsConfiguration {
                domain_tag: "from_configuration_json".to_string(),
                interface_name: Some("Wi-Fi".to_string()),
                fragment_size: 1344
            }
        );
    }
}
