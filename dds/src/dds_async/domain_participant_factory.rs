use alloc::borrow::ToOwned;
use core::ops::DerefMut;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use super::domain_participant::DomainParticipantAsync;
use crate::{
    dcps::{
        channels::oneshot::oneshot,
        dcps_mail::{DcpsMail, ParticipantFactoryMail},
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
    },
    dds_async::{
        configuration::DustDdsConfiguration, domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::{
        domain::DomainId,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::{DdsRuntime, Spawner},
    transport::{
        interface::{TransportDataReceiver, TransportParticipantFactory},
        types::{ENTITYID_PARTICIPANT, Guid, GuidPrefix},
    },
};

const DCPS_CHANNEL_SIZE: usize = 256;

#[doc(hidden)]
pub type DcpsChannel = embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    DcpsMail,
    DCPS_CHANNEL_SIZE,
>;

#[doc(hidden)]
pub type DcpsSender = embassy_sync::channel::Sender<
    'static,
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    DcpsMail,
    DCPS_CHANNEL_SIZE,
>;

#[doc(hidden)]
pub type DcpsReceiver = embassy_sync::channel::Receiver<
    'static,
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    DcpsMail,
    DCPS_CHANNEL_SIZE,
>;

/// Async version of [`DomainParticipantFactory`](crate::domain::domain_participant_factory::DomainParticipantFactory).
/// Unlike the sync version, the [`DomainParticipantFactoryAsync`] is not a singleton and can be created by means of
/// a constructor by passing a DDS runtime. This allows the factory
/// to spin tasks on an existing runtime which can be shared with other things outside Dust DDS.
pub struct DomainParticipantFactoryAsync<T: TransportParticipantFactory> {
    dcps_sender: DcpsSender,
    entity_counter: core::sync::atomic::AtomicU32,
    app_id: [u8; 4],
    host_id: [u8; 4],
    transport: embassy_sync::mutex::Mutex<CriticalSectionRawMutex, T>,
    configuration: embassy_sync::mutex::Mutex<CriticalSectionRawMutex, DustDdsConfiguration>,
}

impl<T: TransportParticipantFactory> DomainParticipantFactoryAsync<T> {
    /// Async version of [`create_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::create_participant).
    pub async fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<impl DomainParticipantListener + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipantAsync> {
        let configuration = self.configuration.lock().await;
        let guid_prefix = self.create_new_guid_prefix();
        let participant_handle = InstanceHandle::from(Guid::new(guid_prefix, ENTITYID_PARTICIPANT));
        let transport_participant = self.transport.lock().await.create_participant(
            domain_id,
            TransportDataReceiver::new(participant_handle, self.dcps_sender),
        );

        let domain_tag = configuration.domain_tag().to_owned();
        let participant_announcement_interval = configuration.participant_announcement_interval();
        let status_kind = mask.to_vec();
        let dcps_listener = a_listener.map(DcpsDomainParticipantListener::new);
        let (reply_sender, reply_receiver) = oneshot();
        self.dcps_sender
            .send(DcpsMail::ParticipantFactory(
                ParticipantFactoryMail::CreateParticipant {
                    guid_prefix,
                    domain_id,
                    qos,
                    dcps_listener,
                    status_kind,
                    reply_sender,
                    transport_participant,
                    domain_tag,
                    participant_announcement_interval,
                },
            ))
            .await;

        let participant_handle = reply_receiver.await??;

        let domain_participant =
            DomainParticipantAsync::new(self.dcps_sender, domain_id, participant_handle);

        Ok(domain_participant)
    }

    /// Async version of [`delete_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::delete_participant).
    pub async fn delete_participant(&self, participant: &DomainParticipantAsync) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        let participant_handle = participant.get_instance_handle();

        self.dcps_sender
            .send(DcpsMail::ParticipantFactory(
                ParticipantFactoryMail::DeleteParticipant {
                    participant_handle,
                    reply_sender,
                },
            ))
            .await;
        reply_receiver.await?
    }

    /// Async version of [`lookup_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::lookup_participant).
    pub async fn lookup_participant(
        &self,
        _domain_id: DomainId,
    ) -> DdsResult<Option<DomainParticipantAsync>> {
        todo!()
    }

    /// Async version of [`set_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_default_participant_qos).
    pub async fn set_default_participant_qos(
        &self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.dcps_sender
            .send(DcpsMail::ParticipantFactory(
                ParticipantFactoryMail::SetDefaultParticipantQos { qos, reply_sender },
            ))
            .await;
        reply_receiver.await?
    }

    /// Async version of [`get_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_default_participant_qos).
    pub async fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.dcps_sender
            .send(DcpsMail::ParticipantFactory(
                ParticipantFactoryMail::GetDefaultParticipantQos { reply_sender },
            ))
            .await;
        reply_receiver.await
    }

    /// Async version of [`set_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_qos).
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let (reply_sender, reply_receiver) = oneshot();
        self.dcps_sender
            .send(DcpsMail::ParticipantFactory(
                ParticipantFactoryMail::SetQos { qos, reply_sender },
            ))
            .await;
        reply_receiver.await?
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_qos).
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        let (reply_sender, reply_receiver) = oneshot();
        self.dcps_sender
            .send(DcpsMail::ParticipantFactory(
                ParticipantFactoryMail::GetQos { reply_sender },
            ))
            .await;
        reply_receiver.await
    }

    /// Get a mutable reference to the transport object
    pub async fn get_mut_transport(&self) -> impl DerefMut<Target = T> + '_ {
        self.transport.lock().await
    }

    /// Get a mutable reference to the configuration object
    pub async fn get_mut_configuration(&self) -> impl DerefMut<Target = DustDdsConfiguration> + '_ {
        self.configuration.lock().await
    }
}

#[cfg(feature = "std")]
impl
    DomainParticipantFactoryAsync<
        crate::rtps_udp_transport::udp_transport::RtpsUdpTransportParticipantFactory,
    >
{
    /// This operation returns the [`DomainParticipantFactoryAsync`] singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same [`DomainParticipantFactoryAsync`] instance.
    #[tracing::instrument]
    pub fn get_instance() -> &'static Self {
        use core::net::IpAddr;
        use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
        use std::sync::OnceLock;
        use tracing::warn;

        static PARTICIPANT_FACTORY_ASYNC: OnceLock<
            DomainParticipantFactoryAsync<
                crate::rtps_udp_transport::udp_transport::RtpsUdpTransportParticipantFactory,
            >,
        > = OnceLock::new();
        PARTICIPANT_FACTORY_ASYNC.get_or_init(|| {

            let executor = crate::std_runtime::executor::Executor::new();
            let timer_driver = crate::std_runtime::timer::TimerDriver::new();
            let runtime = crate::std_runtime::StdRuntime::new(executor, timer_driver);
            let interface_address = NetworkInterface::show()
                .expect("Could not scan interfaces")
                .into_iter()
                .flat_map(|i| {
                    i.addr
                        .into_iter()
                        .filter(|a| matches!(a, Addr::V4(v4) if !v4.ip.is_loopback()))
                })
                .next();
            let host_id = if let Some(interface) = interface_address {
                match interface.ip() {
                    IpAddr::V4(a) => a.octets(),
                    IpAddr::V6(_) => unimplemented!("IPv6 not yet implemented"),
                }
            } else {
                warn!("Failed to get Host ID from IP address, use 0 instead");
                [0; 4]
            };

            let app_id = std::process::id().to_ne_bytes();
            let transport = crate::rtps_udp_transport::udp_transport::RtpsUdpTransportParticipantFactory::default();
            let configuration = Default::default();
            Self::new(runtime, app_id, host_id, transport, configuration)
        })
    }
}

impl<T: TransportParticipantFactory> DomainParticipantFactoryAsync<T> {
    #[doc(hidden)]
    pub fn new<R: DdsRuntime>(
        runtime: R,
        app_id: [u8; 4],
        host_id: [u8; 4],
        transport: T,
        configuration: DustDdsConfiguration,
    ) -> Self {
        static DCPS_CHANNEL: DcpsChannel = DcpsChannel::new();
        let spawner_handle = runtime.spawner();

        let mut domain_participant_factory =
            crate::dcps::dcps_participant_factory::DcpsParticipantFactory::new(
                runtime,
                DCPS_CHANNEL.sender(),
            );
        let dcps_receiver = DCPS_CHANNEL.receiver();
        spawner_handle.spawn(async move {
            loop {
                let m = dcps_receiver.receive().await;
                domain_participant_factory.handle(m);
            }
        });
        Self {
            dcps_sender: DCPS_CHANNEL.sender(),
            app_id,
            host_id,
            entity_counter: core::sync::atomic::AtomicU32::new(0),
            transport: Mutex::new(transport),
            configuration: Mutex::new(configuration),
        }
    }

    fn create_new_guid_prefix(&self) -> GuidPrefix {
        let instance_id = self
            .entity_counter
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed)
            .to_ne_bytes();

        [
            self.host_id[0],
            self.host_id[1],
            self.host_id[2],
            self.host_id[3], // Host ID
            self.app_id[0],
            self.app_id[1],
            self.app_id[2],
            self.app_id[3], // App ID
            instance_id[0],
            instance_id[1],
            instance_id[2],
            instance_id[3], // Instance ID
        ]
    }
}
