use alloc::{boxed::Box, sync::Arc};

use super::domain_participant::DomainParticipantAsync;
use crate::{
    dcps::{
        channels::rpc::{RpcClient, rpc_channel},
        dcps_mail::{CreateParticipantMail, DomainParticipantFactoryMail, DomainParticipantMail},
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
    },
    dds_async::domain_participant_listener::DomainParticipantListener,
    infrastructure::{
        configuration::DustDdsConfiguration,
        domain::DomainId,
        error::DdsResult,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::{Clock, DdsRuntime, Spawner, TaskHandle},
    security::plugins::{
        access_control::AccessControl, authentication::Authentication,
        cryptographic::Cryptographic, types::DdsSecurityPlugins,
    },
    transport::interface::Transport,
};

#[doc(hidden)]
pub type DcpsSender = RpcClient<DomainParticipantMail>;

/// Async version of [`DomainParticipantFactory`](crate::domain::domain_participant_factory::DomainParticipantFactory).
/// Unlike the sync version, the [`DomainParticipantFactoryAsync`] is not a singleton and can be created by means of
/// a constructor by passing a DDS runtime. This allows the factory
/// to spin tasks on an existing runtime which can be shared with other things outside Dust DDS.
pub struct DomainParticipantFactoryAsync {
    dcps_sender: RpcClient<DomainParticipantFactoryMail>,
    worker_task: alloc::boxed::Box<dyn TaskHandle>,
    run_loop: Arc<core::sync::atomic::AtomicBool>,
}

impl DomainParticipantFactoryAsync {
    /// Async version of [`create_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::create_participant).
    pub async fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<impl DomainParticipantListener + Send + 'static>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipantAsync> {
        let listener_mask = mask.iter().collect();
        let dcps_listener = a_listener.map(DcpsDomainParticipantListener::new);

        let reply = self
            .dcps_sender
            .call(DomainParticipantFactoryMail::CreateParticipant(Box::new(
                CreateParticipantMail {
                    domain_id,
                    qos,
                    dcps_listener,
                    listener_mask,
                },
            )))
            .await?;

        let (participant_handle, participant_sender) = reply.expect_created_participant()?;

        let domain_participant =
            DomainParticipantAsync::new(participant_sender, domain_id, participant_handle);

        Ok(domain_participant)
    }

    /// Async version of [`delete_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::delete_participant).
    pub async fn delete_participant(&self, participant: &DomainParticipantAsync) -> DdsResult<()> {
        let participant_handle = participant.get_instance_handle();

        self.dcps_sender
            .call(DomainParticipantFactoryMail::DeleteParticipant { participant_handle })
            .await?
            .expect_ok()
    }

    /// Async version of [`lookup_participant`](crate::domain::domain_participant_factory::DomainParticipantFactory::lookup_participant).
    pub async fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> DdsResult<Option<DomainParticipantAsync>> {
        let reply = self
            .dcps_sender
            .call(DomainParticipantFactoryMail::LookupParticipant { domain_id })
            .await?;
        Ok(reply
            .expect_option_created_participant()?
            .map(|(handle, participant_sender)| {
                DomainParticipantAsync::new(participant_sender, domain_id, handle)
            }))
    }

    /// Async version of [`set_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_default_participant_qos).
    pub async fn set_default_participant_qos(
        &self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        self.dcps_sender
            .call(DomainParticipantFactoryMail::SetDefaultParticipantQos { qos: Box::new(qos) })
            .await?
            .expect_ok()
    }

    /// Async version of [`get_default_participant_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_default_participant_qos).
    pub async fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.dcps_sender
            .call(DomainParticipantFactoryMail::GetDefaultParticipantQos)
            .await?
            .expect_participant_qos()
    }

    /// Async version of [`set_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::set_qos).
    pub async fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        self.dcps_sender
            .call(DomainParticipantFactoryMail::SetQos { qos: Box::new(qos) })
            .await?
            .expect_ok()
    }

    /// Async version of [`get_qos`](crate::domain::domain_participant_factory::DomainParticipantFactory::get_qos).
    pub async fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        self.dcps_sender
            .call(DomainParticipantFactoryMail::GetQos)
            .await?
            .expect_factory_qos()
    }
}

#[cfg(feature = "std")]
impl DomainParticipantFactoryAsync {
    /// This operation returns the [`DomainParticipantFactoryAsync`] singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same [`DomainParticipantFactoryAsync`] instance.
    #[tracing::instrument]
    pub fn get_instance() -> &'static Self {
        Self::get_custom_instance(
            DustDdsConfiguration::default(),
            crate::rtps_udp_transport::udp_transport::RtpsUdpTransport::default(),
            DdsSecurityPlugins::disabled(),
        )
    }

    /// This operation returns the [`DomainParticipantFactoryAsync`] singleton initialized with a custom transport and configuration.
    /// The operation is idempotent, returning the existing instance if it has already been initialized.
    #[tracing::instrument(skip(transport, configuration, security))]
    pub fn get_custom_instance<
        Auth: Authentication,
        Access: AccessControl,
        Crypto: Cryptographic,
    >(
        configuration: DustDdsConfiguration,
        transport: crate::rtps_udp_transport::udp_transport::RtpsUdpTransport,
        security: Option<DdsSecurityPlugins<Auth, Access, Crypto>>,
    ) -> &'static Self {
        use std::sync::OnceLock;

        static PARTICIPANT_FACTORY_ASYNC: OnceLock<DomainParticipantFactoryAsync> = OnceLock::new();
        PARTICIPANT_FACTORY_ASYNC.get_or_init(|| {
            let runtime = crate::std_runtime::StdRuntime::default();
            Self::new(configuration, runtime, transport, security)
        })
    }
}

impl DomainParticipantFactoryAsync {
    #[doc(hidden)]
    pub fn new<
        R: DdsRuntime,
        T: Transport,
        Auth: Authentication,
        Access: AccessControl,
        Crypto: Cryptographic,
    >(
        configuration: DustDdsConfiguration,
        runtime: R,
        mut transport: T,
        mut security: Option<DdsSecurityPlugins<Auth, Access, Crypto>>,
    ) -> Self {
        let clock = runtime.clock();
        let timer = runtime.timer();
        let spawner = runtime.spawner();
        let (dcps_sender, rpc_receiver) = rpc_channel();
        let spawner_handle = spawner.clone();

        let mut domain_participant_factory =
            crate::dcps::dcps_participant_factory::DcpsParticipantFactory::new(configuration);
        let run_loop = Arc::new(core::sync::atomic::AtomicBool::new(true));
        let run_loop_clone = run_loop.clone();
        let clock_clone = clock.clone();
        let timer_clone = timer.clone();
        let spawner_clone = spawner.clone();
        let worker_task = spawner_handle.spawn(async move {
            let span = tracing::trace_span!("dds_factory_loop");
            let _enter = span.enter();
            while run_loop_clone.load(core::sync::atomic::Ordering::Relaxed) {
                let (user_mail, reply_sender) = rpc_receiver.receive().await;
                let now = clock_clone.now();
                let reply = domain_participant_factory
                    .handle_factory_mail(
                        user_mail,
                        now,
                        &timer_clone,
                        &clock_clone,
                        &spawner_clone,
                        &mut security,
                        &mut transport,
                    )
                    .await;
                reply_sender.send(reply);
            }
        });
        Self {
            dcps_sender,
            worker_task: Box::new(worker_task),
            run_loop,
        }
    }

    #[doc(hidden)]
    pub fn shutdown(&self) {
        self.run_loop
            .store(false, core::sync::atomic::Ordering::Relaxed);
        self.worker_task.join();
    }
}
