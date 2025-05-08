use core::net::IpAddr;
use std::sync::OnceLock;

use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};

use super::domain_participant::DomainParticipantAsync;
use crate::{
    configuration::DustDdsConfiguration,
    dcps::{
        domain_participant_actor_mail::{
            DiscoveryServiceMail, DomainParticipantMail, ParticipantServiceMail,
        },
        domain_participant_factory_actor::{
            DdsTransportParticipantFactory, DomainParticipantFactoryActor,
            DomainParticipantFactoryMail,
        },
        listeners::domain_participant_listener::DomainParticipantListenerActor,
        runtime::{ChannelSend, DdsRuntime, OneshotReceive},
    },
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::{actor::Actor, executor::Executor, timer::TimerDriver, StdRuntime},
};
use tracing::warn;

/// Async version of [`DomainParticipantFactory`](crate::domain::domain_participant_factory::DomainParticipantFactory).
/// Unlike the sync version, the [`DomainParticipantFactoryAsync`] is not a singleton and can be created by means of
/// a constructor by passing a DDS runtime. This allows the factory
/// to spin tasks on an existing runtime which can be shared with other things outside Dust DDS.
pub struct DomainParticipantFactoryAsync<R: DdsRuntime> {
    runtime: R,
    domain_participant_factory_actor: Actor<R, DomainParticipantFactoryActor<R>>,
}

impl<R: DdsRuntime> DomainParticipantFactoryAsync<R> {
    /// Async version of [`create_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::create_participant).
    pub async fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: impl DomainParticipantListener<R> + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipantAsync<R>> {
        let clock_handle = self.runtime.clock();
        let timer_handle = self.runtime.timer();
        let spawner_handle = self.runtime.spawner();
        let status_kind = mask.to_vec();
        let listener_sender =
            DomainParticipantListenerActor::spawn::<R>(a_listener, &spawner_handle);
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.domain_participant_factory_actor
            .send_actor_mail(DomainParticipantFactoryMail::CreateParticipant {
                domain_id,
                qos,
                listener_sender,
                status_kind,
                reply_sender,
                clock_handle,
                timer_handle,
                spawner_handle: spawner_handle.clone(),
            })
            .await;

        let (
            participant_address,
            participant_handle,
            participant_status_condition_address,
            builtin_subscriber_status_condition_address,
        ) = reply_receiver.receive().await??;

        let domain_participant = DomainParticipantAsync::new(
            participant_address.clone(),
            participant_status_condition_address,
            builtin_subscriber_status_condition_address,
            domain_id,
            participant_handle,
            spawner_handle,
        );

        Ok(domain_participant)
    }

    /// Async version of [`delete_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::delete_participant).
    pub async fn delete_participant(
        &self,
        participant: &DomainParticipantAsync<R>,
    ) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        participant
            .participant_address()
            .send(DomainParticipantMail::Participant(
                ParticipantServiceMail::IsEmpty { reply_sender },
            ))
            .await?;
        let is_participant_empty = reply_receiver.receive().await?;
        if is_participant_empty {
            let (reply_sender, mut reply_receiver) = R::oneshot();
            let handle = participant.get_instance_handle().await;

            self.domain_participant_factory_actor
                .send_actor_mail(DomainParticipantFactoryMail::DeleteParticipant {
                    handle,
                    reply_sender,
                })
                .await;
            let deleted_participant = reply_receiver.receive().await??;
            deleted_participant
                .send(DomainParticipantMail::Discovery(
                    DiscoveryServiceMail::AnnounceDeletedParticipant,
                ))
                .await
                .ok();
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    /// Async version of [`lookup_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::lookup_participant).
    pub async fn lookup_participant(
        &self,
        _domain_id: DomainId,
    ) -> DdsResult<Option<DomainParticipantAsync<R>>> {
        todo!()
    }

    /// Async version of [`set_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_default_participant_qos).
    pub async fn set_default_participant_qos(
        &self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.domain_participant_factory_actor
            .send_actor_mail(DomainParticipantFactoryMail::SetDefaultParticipantQos {
                qos,
                reply_sender,
            })
            .await;
        reply_receiver.receive().await?
    }

    /// Async version of [`get_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_default_participant_qos).
    pub async fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.domain_participant_factory_actor
            .send_actor_mail(DomainParticipantFactoryMail::GetDefaultParticipantQos {
                reply_sender,
            })
            .await;
        Ok(reply_receiver.receive().await?)
    }

    /// Async version of [`set_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_qos).
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.domain_participant_factory_actor
            .send_actor_mail(DomainParticipantFactoryMail::SetQos { qos, reply_sender })
            .await;
        reply_receiver.receive().await?
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_qos).
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.domain_participant_factory_actor
            .send_actor_mail(DomainParticipantFactoryMail::GetQos { reply_sender })
            .await;
        Ok(reply_receiver.receive().await?)
    }

    /// Async version of [`set_configuration`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_configuration).
    pub async fn set_configuration(&self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_actor_mail(DomainParticipantFactoryMail::SetConfiguration { configuration })
            .await;
        Ok(())
    }

    /// Async version of [`get_configuration`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_configuration).
    pub async fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        let (reply_sender, mut reply_receiver) = R::oneshot();
        self.domain_participant_factory_actor
            .send_actor_mail(DomainParticipantFactoryMail::GetConfiguration { reply_sender })
            .await;
        Ok(reply_receiver.receive().await?)
    }

    /// Async version of [`set_transport`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_transport).
    pub async fn set_transport(&self, transport: DdsTransportParticipantFactory) -> DdsResult<()> {
        self.domain_participant_factory_actor
            .send_actor_mail(DomainParticipantFactoryMail::SetTransport { transport })
            .await;
        Ok(())
    }
}

impl<R: DdsRuntime> DomainParticipantFactoryAsync<R> {
    #[doc(hidden)]
    pub fn new(runtime: R, app_id: [u8; 4], host_id: [u8; 4]) -> DomainParticipantFactoryAsync<R> {
        let domain_participant_factory_actor = Actor::spawn(
            DomainParticipantFactoryActor::new(app_id, host_id),
            &runtime.spawner(),
        );
        DomainParticipantFactoryAsync {
            runtime,
            domain_participant_factory_actor,
        }
    }
}

impl DomainParticipantFactoryAsync<StdRuntime> {
    /// This operation returns the [`DomainParticipantFactoryAsync`] singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same [`DomainParticipantFactoryAsync`] instance.
    #[tracing::instrument]
    pub fn get_instance() -> &'static DomainParticipantFactoryAsync<StdRuntime> {
        static PARTICIPANT_FACTORY_ASYNC: OnceLock<DomainParticipantFactoryAsync<StdRuntime>> =
            OnceLock::new();
        PARTICIPANT_FACTORY_ASYNC.get_or_init(|| {
            let executor = Executor::new();
            let timer_driver = TimerDriver::new();
            let runtime = StdRuntime::new(executor, timer_driver);
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
            DomainParticipantFactoryAsync::new(runtime, app_id, host_id)
        })
    }
}
