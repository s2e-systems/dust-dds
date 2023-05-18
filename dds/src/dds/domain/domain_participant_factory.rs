use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    str::FromStr,
    sync::atomic::{self, AtomicU32},
};

use super::domain_participant::{
    task_metatraffic_multicast_receive, task_metatraffic_unicast_receive,
    task_send_entity_announce, task_unicast_metatraffic_communication_send,
    task_unicast_user_defined_communication_send, task_user_defined_receive, DomainParticipant,
};
use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        configuration::DustDdsConfiguration,
        dds::{
            any_data_reader_listener::AnyDataReaderListener,
            any_data_writer_listener::AnyDataWriterListener, any_topic_listener::AnyTopicListener,
            dds_domain_participant::DdsDomainParticipant, nodes::DomainParticipantNode,
            status_listener::StatusListener,
        },
        rtps::{
            participant::RtpsParticipant,
            types::{
                Guid, GuidPrefix, Locator, LocatorAddress, LocatorPort, LOCATOR_KIND_UDP_V4,
                PROTOCOLVERSION, VENDOR_ID_S2E,
            },
        },
        rtps_udp_psm::udp_transport::UdpTransportRead,
        utils::{condvar::DdsCondvar, shared_object::DdsRwLock},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    publication::publisher_listener::PublisherListener,
    subscription::subscriber_listener::SubscriberListener,
};

use jsonschema::JSONSchema;
use lazy_static::lazy_static;
use mac_address::MacAddress;
use schemars::schema_for;
use socket2::Socket;

pub type DomainId = i32;

/// This value can be used as an alias for the singleton factory returned by the operation
/// [`DomainParticipantFactory::get_instance()`].
pub static THE_PARTICIPANT_FACTORY: DomainParticipantFactory = DomainParticipantFactory;

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

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// [`DomainParticipantFactory`] itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory;

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
        let dp = THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .create_participant(domain_id, qos, a_listener, mask)?;

        if THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .qos()
            .entity_factory
            .autoenable_created_entities
        {
            dp.enable()?;
        }

        Ok(dp)
    }

    /// This operation deletes an existing [`DomainParticipant`]. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted otherwise the error [`DdsError::PreconditionNotMet`] is returned. If the
    /// participant has been previously deleted this operation returns the error [`DdsError::AlreadyDeleted`].
    pub fn delete_participant(&self, participant: &DomainParticipant) -> DdsResult<()> {
        let is_participant_empty = THE_DDS_DOMAIN_PARTICIPANT_FACTORY.get_participant(
            &participant.node().guid().prefix(),
            |dp| {
                let dp = dp.ok_or(DdsError::AlreadyDeleted)?;
                Ok(dp.user_defined_publisher_list().iter().count() == 0
                    && dp.user_defined_subscriber_list().iter().count() == 0
                    && dp.topic_list().iter().count() == 0)
            },
        )?;

        if is_participant_empty {
            // Explicit external drop to avoid deadlock. Otherwise objects contained in the factory can't access it due to it being blocked
            // while locking
            let object = THE_DDS_DOMAIN_PARTICIPANT_FACTORY
                .domain_participant_list
                .write_lock()
                .remove(&participant.node().guid().prefix());
            std::mem::drop(object);

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
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .domain_participant_list
            .read_lock()
            .iter()
            .find_map(|(_, dp)| {
                if dp.get_domain_id() == domain_id {
                    Some(dp.guid())
                } else {
                    None
                }
            })
            .map(|guid| DomainParticipant::new(DomainParticipantNode::new(guid)))
    }

    /// This operation sets a default value of the [`DomainParticipantQos`] policies which will be used for newly created
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_default_participant_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let q = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.set_default_participant_qos(q);

        Ok(())
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`]
    /// operation.
    /// The values retrieved by [`DomainParticipantFactory::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`DomainParticipantFactory::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    pub fn get_default_participant_qos(&self) -> DomainParticipantQos {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.default_participant_qos()
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let q = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.set_qos(q);

        Ok(())
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    pub fn get_qos(&self) -> DomainParticipantFactoryQos {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY.qos()
    }
}

pub struct DdsDomainParticipantFactory {
    domain_participant_list: DdsRwLock<HashMap<GuidPrefix, DdsDomainParticipant>>,
    domain_participant_counter: AtomicU32,
    domain_participant_listener_list:
        DdsRwLock<HashMap<Guid, StatusListener<dyn DomainParticipantListener + Send + Sync>>>,
    publisher_listener_list:
        DdsRwLock<HashMap<Guid, StatusListener<dyn PublisherListener + Send + Sync>>>,
    subscriber_listener_list:
        DdsRwLock<HashMap<Guid, StatusListener<dyn SubscriberListener + Send + Sync>>>,
    topic_listener_list:
        DdsRwLock<HashMap<Guid, StatusListener<dyn AnyTopicListener + Send + Sync>>>,
    data_reader_listener_list:
        DdsRwLock<HashMap<Guid, StatusListener<dyn AnyDataReaderListener + Send + Sync>>>,
    data_writer_listener_list:
        DdsRwLock<HashMap<Guid, StatusListener<dyn AnyDataWriterListener + Send + Sync>>>,
    qos: DdsRwLock<DomainParticipantFactoryQos>,
    default_participant_qos: DdsRwLock<DomainParticipantQos>,
}

impl Default for DdsDomainParticipantFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DdsDomainParticipantFactory {
    pub fn new() -> Self {
        Self {
            domain_participant_list: DdsRwLock::new(HashMap::new()),
            domain_participant_counter: AtomicU32::new(0),
            domain_participant_listener_list: DdsRwLock::new(HashMap::new()),
            publisher_listener_list: DdsRwLock::new(HashMap::new()),
            subscriber_listener_list: DdsRwLock::new(HashMap::new()),
            topic_listener_list: DdsRwLock::new(HashMap::new()),
            data_reader_listener_list: DdsRwLock::new(HashMap::new()),
            data_writer_listener_list: DdsRwLock::new(HashMap::new()),
            qos: DdsRwLock::new(DomainParticipantFactoryQos::default()),
            default_participant_qos: DdsRwLock::new(DomainParticipantQos::default()),
        }
    }

    pub fn qos(&self) -> DomainParticipantFactoryQos {
        self.qos.read_lock().clone()
    }

    pub fn set_qos(&self, qos: DomainParticipantFactoryQos) {
        *self.qos.write_lock() = qos;
    }

    pub fn default_participant_qos(&self) -> DomainParticipantQos {
        self.default_participant_qos.read_lock().clone()
    }

    pub fn set_default_participant_qos(&self, default_participant_qos: DomainParticipantQos) {
        *self.default_participant_qos.write_lock() = default_participant_qos;
    }

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
            QosKind::Default => THE_DDS_DOMAIN_PARTICIPANT_FACTORY.default_participant_qos(),
            QosKind::Specific(q) => q,
        };

        let interface_address_list =
            get_interface_address_list(configuration.interface_name.as_ref());

        let default_unicast_socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
            .map_err(|_| DdsError::Error)?;
        default_unicast_socket.set_nonblocking(true).unwrap();
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
        metattrafic_unicast_socket.set_nonblocking(true).unwrap();

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

        let spdp_discovery_locator_list = metatraffic_multicast_locator_list.clone();

        let mac_address = ifcfg::IfCfg::get()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter_map(|i| MacAddress::from_str(&i.mac).ok())
            .find(|&mac| mac != MacAddress::new([0, 0, 0, 0, 0, 0]))
            .expect("Could not find any mac address")
            .bytes();

        let app_id = std::process::id().to_ne_bytes();
        let instance_id = self
            .domain_participant_counter
            .fetch_add(1, atomic::Ordering::SeqCst)
            .to_ne_bytes();

        #[rustfmt::skip]
        let guid_prefix = GuidPrefix::new([
            mac_address[2],  mac_address[3], mac_address[4], mac_address[5], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
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
        let guid = rtps_participant.guid();
        let sedp_condvar = DdsCondvar::new();
        let user_defined_data_send_condvar = DdsCondvar::new();
        let (announce_sender, announce_receiver) = tokio::sync::mpsc::channel(500);

        let dds_participant = DdsDomainParticipant::new(
            rtps_participant,
            domain_id,
            configuration.domain_tag,
            domain_participant_qos,
            &spdp_discovery_locator_list,
            user_defined_data_send_condvar.clone(),
            configuration.fragment_size,
            announce_sender,
            sedp_condvar.clone(),
        );

        dds_participant.spawn(task_send_entity_announce(guid_prefix, announce_receiver));

        self.add_domain_participant_listener(guid, a_listener, mask);

        let enter_guard = dds_participant.task_executor().enter();
        let (listener_sender, listener_receiver) = tokio::sync::mpsc::channel(500);
        std::mem::drop(enter_guard);

        dds_participant.spawn(
            crate::domain::domain_participant::task_update_communication_status(
                guid_prefix,
                listener_sender.clone(),
            ),
        );

        dds_participant.spawn(crate::domain::domain_participant::task_listener_receiver(
            listener_receiver,
        ));

        let enter_guard = dds_participant.task_executor().enter();
        let metatraffic_multicast_transport = UdpTransportRead::new(
            get_multicast_socket(
                DEFAULT_MULTICAST_LOCATOR_ADDRESS,
                port_builtin_multicast(domain_id),
            )
            .unwrap(),
        );
        std::mem::drop(enter_guard);
        dds_participant.spawn(task_metatraffic_multicast_receive(
            guid_prefix,
            metatraffic_multicast_transport,
            dds_participant.sedp_condvar().clone(),
            listener_sender.clone(),
        ));

        let enter_guard = dds_participant.task_executor().enter();
        let metatraffic_unicast_transport = UdpTransportRead::new(
            tokio::net::UdpSocket::from_std(metattrafic_unicast_socket).unwrap(),
        );
        std::mem::drop(enter_guard);

        dds_participant.spawn(task_metatraffic_unicast_receive(
            guid_prefix,
            metatraffic_unicast_transport,
            dds_participant.sedp_condvar().clone(),
            listener_sender.clone(),
        ));

        let enter_guard = dds_participant.task_executor().enter();
        let default_unicast_transport =
            UdpTransportRead::new(tokio::net::UdpSocket::from_std(default_unicast_socket).unwrap());
        std::mem::drop(enter_guard);

        dds_participant.spawn(task_user_defined_receive(
            guid_prefix,
            default_unicast_transport,
            listener_sender,
        ));

        dds_participant.spawn(task_unicast_metatraffic_communication_send(
            guid_prefix,
            sedp_condvar,
        ));

        dds_participant.spawn(task_unicast_user_defined_communication_send(
            guid_prefix,
            user_defined_data_send_condvar,
        ));

        self.domain_participant_list
            .write_lock()
            .insert(guid_prefix, dds_participant);

        Ok(DomainParticipant::new(DomainParticipantNode::new(guid)))
    }

    pub fn remove_participant(&self, guid_prefix: &GuidPrefix) {
        self.domain_participant_list
            .write_lock()
            .remove(guid_prefix);
    }

    pub fn get_participant<F, O>(&self, guid_prefix: &GuidPrefix, f: F) -> O
    where
        F: FnOnce(Option<&DdsDomainParticipant>) -> O,
    {
        f(self.domain_participant_list.read_lock().get(guid_prefix))
    }

    pub fn get_participant_mut<F, O>(&self, guid_prefix: &GuidPrefix, f: F) -> O
    where
        F: FnOnce(Option<&mut DdsDomainParticipant>) -> O,
    {
        f(self
            .domain_participant_list
            .write_lock()
            .get_mut(guid_prefix))
    }

    pub fn add_domain_participant_listener(
        &self,
        domain_participant_guid: Guid,
        listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        self.domain_participant_listener_list
            .write_lock()
            .insert(domain_participant_guid, StatusListener::new(listener, mask));
    }

    pub fn delete_domain_participant_listener(&self, domain_participant_guid: &Guid) {
        self.domain_participant_listener_list
            .write_lock()
            .remove(domain_participant_guid);
    }

    pub fn get_domain_participant_listener<F, O>(&self, domain_participant_guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut StatusListener<dyn DomainParticipantListener + Send + Sync>>) -> O,
    {
        f(self
            .domain_participant_listener_list
            .write_lock()
            .get_mut(domain_participant_guid))
    }

    pub fn add_subscriber_listener(
        &self,
        subscriber_guid: Guid,
        listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        self.subscriber_listener_list
            .write_lock()
            .insert(subscriber_guid, StatusListener::new(listener, mask));
    }

    pub fn delete_subscriber_listener(&self, subscriber_guid: &Guid) {
        self.subscriber_listener_list
            .write_lock()
            .remove(subscriber_guid);
    }

    pub fn get_subscriber_listener<F, O>(&self, subscriber_guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut StatusListener<dyn SubscriberListener + Send + Sync>>) -> O,
    {
        f(self
            .subscriber_listener_list
            .write_lock()
            .get_mut(subscriber_guid))
    }

    pub fn add_publisher_listener(
        &self,
        publisher_guid: Guid,
        listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        self.publisher_listener_list
            .write_lock()
            .insert(publisher_guid, StatusListener::new(listener, mask));
    }

    pub fn delete_publisher_listener(&self, publisher_guid: &Guid) {
        self.publisher_listener_list
            .write_lock()
            .remove(publisher_guid);
    }

    pub fn get_publisher_listener<F, O>(&self, publisher_guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut StatusListener<dyn PublisherListener + Send + Sync>>) -> O,
    {
        f(self
            .publisher_listener_list
            .write_lock()
            .get_mut(publisher_guid))
    }

    pub fn add_topic_listener(
        &self,
        topic_guid: Guid,
        listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        self.topic_listener_list
            .write_lock()
            .insert(topic_guid, StatusListener::new(listener, mask));
    }

    pub fn delete_topic_listener(&self, topic_guid: &Guid) {
        self.topic_listener_list.write_lock().remove(topic_guid);
    }

    pub fn get_topic_listener<F, O>(&self, topic_guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut StatusListener<dyn AnyTopicListener + Send + Sync>>) -> O,
    {
        f(self.topic_listener_list.write_lock().get_mut(topic_guid))
    }

    pub fn add_data_reader_listener(
        &self,
        data_reader_guid: Guid,
        listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        self.data_reader_listener_list
            .write_lock()
            .insert(data_reader_guid, StatusListener::new(listener, mask));
    }

    pub fn delete_data_reader_listener(&self, data_reader_guid: &Guid) {
        self.data_reader_listener_list
            .write_lock()
            .remove(data_reader_guid);
    }

    pub fn get_data_reader_listener<F, O>(&self, data_reader_guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut StatusListener<dyn AnyDataReaderListener + Send + Sync>>) -> O,
    {
        f(self
            .data_reader_listener_list
            .write_lock()
            .get_mut(data_reader_guid))
    }

    pub fn add_data_writer_listener(
        &self,
        data_writer_guid: Guid,
        listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        self.data_writer_listener_list
            .write_lock()
            .insert(data_writer_guid, StatusListener::new(listener, mask));
    }

    pub fn delete_data_writer_listener(&self, data_writer_guid: &Guid) {
        self.data_writer_listener_list
            .write_lock()
            .remove(data_writer_guid);
    }

    pub fn get_data_writer_listener<F, O>(&self, data_writer_guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut StatusListener<dyn AnyDataWriterListener + Send + Sync>>) -> O,
    {
        f(self
            .data_writer_listener_list
            .write_lock()
            .get_mut(data_writer_guid))
    }
}

lazy_static! {
    pub(in crate::dds) static ref THE_DDS_DOMAIN_PARTICIPANT_FACTORY: DdsDomainParticipantFactory =
        DdsDomainParticipantFactory::new();
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
