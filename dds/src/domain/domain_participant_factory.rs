use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    str::FromStr,
};

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        configuration::DustDdsConfiguration,
        dds_impl::{dcps_service::DcpsService, domain_participant_impl::DomainParticipantImpl},
        rtps::{
            participant::RtpsParticipant,
            types::{
                GuidPrefix, Locator, LocatorAddress, LocatorPort, LOCATOR_KIND_UDP_V4,
                PROTOCOLVERSION, VENDOR_ID_S2E,
            },
        },
        rtps_udp_psm::udp_transport::UdpTransport,
        utils::{condvar::DdsCondvar, shared_object::DdsRwLock},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};
use jsonschema::JSONSchema;
use lazy_static::lazy_static;
use mac_address::MacAddress;
use schemars::schema_for;
use socket2::Socket;

use super::domain_participant::DomainParticipant;

pub type DomainId = i32;

lazy_static! {
    /// This value can be used as an alias for the singleton factory returned by the operation
    /// [`DomainParticipantFactory::get_instance()`].
    pub static ref THE_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory {
        participant_list: DdsRwLock::new(vec![]),
        qos: DdsRwLock::new(DomainParticipantFactoryQos::default()),
        default_participant_qos: DdsRwLock::new(DomainParticipantQos::default()),
    };
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

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// [`DomainParticipantFactory`] itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory {
    participant_list: DdsRwLock<Vec<DcpsService>>,
    qos: DdsRwLock<DomainParticipantFactoryQos>,
    default_participant_qos: DdsRwLock<DomainParticipantQos>,
}

impl DomainParticipantFactory {
    /// This operation creates a new [`DomainParticipant`] object. The [`DomainParticipant`] signifies that the calling application intends
    /// to join the Domain identified by the `domain_id` argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no [`DomainParticipant`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`DomainParticipant`] should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation [`DomainParticipantFactory::get_default_participant_qos`] and using the resulting
    /// QoS to create the [`DomainParticipant`].
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant> {
        let configuration = if let Ok(configuration_json) = std::env::var("DUST_DDS_CONFIGURATION")
        {
            configuration_try_from_str(configuration_json.as_str())
                .map_err(DdsError::PreconditionNotMet)?
        } else {
            DustDdsConfiguration::default()
        };

        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.read_lock().clone(),
            QosKind::Specific(q) => q,
        };

        let interface_address_list =
            get_interface_address_list(configuration.interface_name.as_ref());

        let default_unicast_socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
            .map_err(|_| DdsError::Error)?;
        let user_defined_unicast_port = default_unicast_socket
            .local_addr()
            .map_err(|_| DdsError::Error)?
            .port();
        let user_defined_unicast_locator_port = LocatorPort::new(user_defined_unicast_port.into());

        let default_unicast_locator_list: Vec<Locator> = interface_address_list
            .iter()
            .map(|a| Locator::new(LOCATOR_KIND_UDP_V4, user_defined_unicast_locator_port, *a))
            .collect();

        let default_multicast_locator_list = vec![];

        let metattrafic_unicast_socket =
            UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
                .map_err(|_| DdsError::Error)?;
        let metattrafic_unicast_locator_port = LocatorPort::new(
            metattrafic_unicast_socket
                .local_addr()
                .map_err(|_| DdsError::Error)?
                .port()
                .into(),
        );
        let metatraffic_unicast_locator_list: Vec<Locator> = interface_address_list
            .iter()
            .map(|a| Locator::new(LOCATOR_KIND_UDP_V4, metattrafic_unicast_locator_port, *a))
            .collect();

        let metatraffic_multicast_locator_list = vec![Locator::new(
            LOCATOR_KIND_UDP_V4,
            port_builtin_multicast(domain_id),
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        )];

        let metatraffic_multicast_transport = UdpTransport::new(
            get_multicast_socket(
                DEFAULT_MULTICAST_LOCATOR_ADDRESS,
                port_builtin_multicast(domain_id),
            )
            .unwrap(),
        );

        let metatraffic_unicast_transport = UdpTransport::new(metattrafic_unicast_socket);

        let default_unicast_transport = UdpTransport::new(default_unicast_socket);

        let spdp_discovery_locator_list = metatraffic_multicast_locator_list.clone();

        let mac_address = ifcfg::IfCfg::get()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter_map(|i| MacAddress::from_str(&i.mac).ok())
            .find(|&mac| mac != MacAddress::new([0, 0, 0, 0, 0, 0]))
            .expect("Could not find any mac address")
            .bytes();

        #[rustfmt::skip]
        let guid_prefix = GuidPrefix::new([
            mac_address[0], mac_address[1], mac_address[2],
            mac_address[3], mac_address[4], mac_address[5],
            domain_id as u8, (user_defined_unicast_port >> 8) as u8, (user_defined_unicast_port & 0x00FF) as u8, 0, 0, 0
        ]);

        let rtps_participant = RtpsParticipant::new(
            guid_prefix,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
        );
        let sedp_condvar = DdsCondvar::new();
        let user_defined_data_send_condvar = DdsCondvar::new();
        let (announce_sender, announce_receiver) = std::sync::mpsc::sync_channel(1);
        let participant = DomainParticipantImpl::new(
            rtps_participant,
            domain_id,
            configuration.domain_tag,
            domain_participant_qos,
            a_listener,
            mask,
            &spdp_discovery_locator_list,
            sedp_condvar,
            user_defined_data_send_condvar,
            configuration.fragment_size,
            announce_sender.clone(),
        );

        let dcps_service = DcpsService::new(
            participant,
            metatraffic_multicast_transport,
            metatraffic_unicast_transport,
            default_unicast_transport,
            announce_sender,
            announce_receiver,
        )?;

        let participant = DomainParticipant::new(dcps_service.participant().downgrade());

        if self
            .qos
            .read_lock()
            .entity_factory
            .autoenable_created_entities
        {
            participant.enable()?;
        }

        self.participant_list.write_lock().push(dcps_service);

        Ok(participant)
    }

    /// This operation deletes an existing [`DomainParticipant`]. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted otherwise the error [`DdsError::PreconditionNotMet`] is returned. If the
    /// participant has been previously deleted this operation returns the error [`DdsError::AlreadyDeleted`].
    pub fn delete_participant(&self, participant: &DomainParticipant) -> DdsResult<()> {
        let mut participant_list = THE_PARTICIPANT_FACTORY.participant_list.write_lock();

        let index = participant_list
            .iter()
            .position(|pm| DomainParticipant::new(pm.participant().downgrade()).eq(participant))
            .ok_or(DdsError::AlreadyDeleted)?;

        if participant_list[index].participant().is_empty() {
            participant_list[index].participant().cancel_timers();
            participant_list[index].shutdown_tasks();
            participant_list.remove(index);

            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    /// This operation returns the [`DomainParticipantFactory`] singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same [`DomainParticipantFactory`] instance.
    /// The pre-defined value [`struct@THE_PARTICIPANT_FACTORY`] can also be used as an alias for the singleton factory returned by this operation.
    pub fn get_instance() -> &'static Self {
        &THE_PARTICIPANT_FACTORY
    }

    /// This operation retrieves a previously created [`DomainParticipant`] belonging to the specified domain_id. If no such
    /// [`DomainParticipant`] exists, the operation will return a [`None`] value.
    /// If multiple [`DomainParticipant`] entities belonging to that domain_id exist, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_participant(&self, domain_id: DomainId) -> Option<DomainParticipant> {
        self.participant_list
            .read_lock()
            .iter()
            .find(|dp| dp.participant().get_domain_id() == domain_id)
            .map(|dp| DomainParticipant::new(dp.participant().downgrade()))
    }

    /// This operation sets a default value of the [`DomainParticipantQos`] policies which will be used for newly created
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_default_participant_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => {
                *self.default_participant_qos.write_lock() = DomainParticipantQos::default()
            }
            QosKind::Specific(q) => *self.default_participant_qos.write_lock() = q,
        };

        Ok(())
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`]
    /// operation.
    /// The values retrieved by [`DomainParticipantFactory::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`DomainParticipantFactory::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    pub fn get_default_participant_qos(&self) -> DomainParticipantQos {
        self.default_participant_qos.read_lock().clone()
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => *self.qos.write_lock() = DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => *self.qos.write_lock() = q,
        }

        Ok(())
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    pub fn get_qos(&self) -> DomainParticipantFactoryQos {
        self.qos.read_lock().clone()
    }
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
